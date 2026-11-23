//! Liability use cases, including the advisor/client staging rules.
//!
//! Advisors change the committed record directly. A client's PATCH or DELETE is staged for
//! advisor review, and a client's POST is created flagged `create`.

use chrono::Utc;
use futures_util::TryStreamExt;
use indexmap::IndexMap;
use mongodb::bson::{Document, doc, oid::ObjectId};
use rust_decimal::Decimal;

use crate::api::caller::CallerContext;
use crate::codec;
use crate::db::Store;
use crate::domain::enums::{CallerType, LiabilityType, StagedAction};
use crate::domain::liability::{AuditStamp, Liability, LiabilityFields, StagedChange};
use crate::domain::patch::LiabilityPatch;
use crate::dto::{
    AdvisorLiabilityView, ClientLiabilityView, FieldChange, LiabilityList, LiabilitySummary,
    LiabilityView, PendingChange,
};
use crate::error::AppError;

// ---- views (pure) ----

/// The staged edits as a typed patch; empty unless the staged action is `update`.
fn staged_patch(doc: &Liability) -> LiabilityPatch {
    match &doc.staged {
        Some(s) if s.action == StagedAction::Update => LiabilityPatch::from_staged(&s.changes),
        _ => LiabilityPatch::default(),
    }
}

/// The fields as a client sees them: committed values overlaid with staged edits.
fn effective(doc: &Liability) -> LiabilityFields {
    let mut fields = doc.fields.clone();
    staged_patch(doc).apply(&mut fields);
    fields
}

fn pending(doc: &Liability) -> Option<PendingChange> {
    let staged = doc.staged.as_ref()?;
    let patch = staged_patch(doc);
    if staged.action == StagedAction::Update && patch.is_empty() {
        return None;
    }
    Some(PendingChange {
        action: staged.action,
        changes: patch
            .diff(&doc.fields)
            .into_iter()
            .map(|(field, current, proposed)| (field.to_owned(), FieldChange { current, proposed }))
            .collect::<IndexMap<_, _>>(),
        created_by: staged.created_by.clone(),
        updated_by: staged.updated_by.clone(),
    })
}

/// Clients get their updated view (committed + staged values) with no change info. Advisors
/// get the committed values plus `pending_change` describing what is staged.
pub fn to_view(doc: &Liability, caller: &CallerContext) -> LiabilityView {
    let id = doc.id.map(|id| id.to_hex()).unwrap_or_default();
    let view = |fields: LiabilityFields| ClientLiabilityView {
        id: id.clone(),
        client_id: doc.client_id.clone(),
        fields,
        created_at: doc.created_at,
        updated_at: doc.updated_at,
        created_by: doc.created_by.clone(),
        updated_by: doc.updated_by.clone(),
    };
    match caller.caller_type {
        CallerType::Client => LiabilityView::Client(view(effective(doc))),
        CallerType::Advisor => LiabilityView::Advisor(AdvisorLiabilityView {
            base: view(doc.fields.clone()),
            pending_change: pending(doc),
        }),
    }
}

/// A client's pending delete hides the record from that client.
fn hidden_from(doc: &Liability, caller: &CallerContext) -> bool {
    caller.caller_type == CallerType::Client && doc.staged_action() == Some(StagedAction::Delete)
}

fn parse_id(liability_id: &str) -> Option<ObjectId> {
    ObjectId::parse_str(liability_id).ok()
}

// ---- persistence ----

impl Store {
    async fn save(&self, doc: &Liability) -> Result<(), AppError> {
        let id = doc
            .id
            .ok_or_else(|| AppError::Codec("cannot save a liability without an id".into()))?;
        self.coll
            .replace_one(doc! { "_id": id }, codec::encode(doc)?)
            .await?;
        Ok(())
    }

    async fn find_all(&self, filter: Document) -> Result<Vec<Liability>, AppError> {
        let raw: Vec<Document> = self.coll.find(filter).await?.try_collect().await?;
        raw.into_iter().map(codec::decode).collect()
    }

    pub async fn create(
        &self,
        client_id: &str,
        fields: LiabilityFields,
        caller: &CallerContext,
    ) -> Result<Liability, AppError> {
        let stamp = caller.stamp();
        // A client's new liability is a proposal until an advisor reviews it.
        let staged = (caller.caller_type == CallerType::Client).then(|| StagedChange {
            action: StagedAction::Create,
            changes: Default::default(),
            created_by: stamp.clone(),
            updated_by: stamp.clone(),
        });
        let mut doc = Liability {
            id: None,
            client_id: client_id.to_owned(),
            fields,
            created_at: stamp.at,
            updated_at: stamp.at,
            created_by: stamp.clone(),
            updated_by: stamp,
            staged,
        };
        let inserted = self.coll.insert_one(codec::encode(&doc)?).await?;
        doc.id = inserted.inserted_id.as_object_id();
        Ok(doc)
    }

    pub async fn list_for_client(
        &self,
        client_id: &str,
        liability_type: Option<LiabilityType>,
        skip: u64,
        limit: i64,
        caller: &CallerContext,
    ) -> Result<LiabilityList, AppError> {
        let mut filter = doc! { "client_id": client_id };
        let is_client = caller.caller_type == CallerType::Client;
        if is_client {
            filter.insert(
                "staged.action",
                doc! { "$ne": StagedAction::Delete.as_str() },
            );
        }
        if let Some(t) = liability_type {
            if is_client {
                // Filter on the type the client sees (staged value wins over committed).
                filter.insert(
                    "$or",
                    vec![
                        doc! { "liability_type": t.as_str(), "staged.changes.liability_type": { "$exists": false } },
                        doc! { "staged.changes.liability_type": t.as_str() },
                    ],
                );
            } else {
                filter.insert("liability_type", t.as_str());
            }
        }
        let total = self.coll.count_documents(filter.clone()).await?;
        let raw: Vec<Document> = self
            .coll
            .find(filter)
            .sort(doc! { "created_at": -1 })
            .skip(skip)
            .limit(limit)
            .await?
            .try_collect()
            .await?;
        let items = raw
            .into_iter()
            .map(|d| codec::decode(d).map(|l| to_view(&l, caller)))
            .collect::<Result<_, _>>()?;
        Ok(LiabilityList { items, total })
    }

    pub async fn get(
        &self,
        client_id: &str,
        liability_id: &str,
        caller: &CallerContext,
    ) -> Result<Option<Liability>, AppError> {
        let Some(oid) = parse_id(liability_id) else {
            return Ok(None);
        };
        let found = self
            .coll
            .find_one(doc! { "_id": oid, "client_id": client_id })
            .await?;
        let doc = found.map(codec::decode).transpose()?;
        Ok(doc.filter(|d| !hidden_from(d, caller)))
    }

    /// Advisor edits apply immediately; client edits are staged for advisor review.
    pub async fn update(
        &self,
        client_id: &str,
        liability_id: &str,
        changes: LiabilityPatch,
        caller: &CallerContext,
    ) -> Result<Option<Liability>, AppError> {
        let Some(mut doc) = self.get(client_id, liability_id, caller).await? else {
            return Ok(None);
        };
        if caller.caller_type == CallerType::Client
            && doc.staged_action() != Some(StagedAction::Create)
        {
            self.stage(&mut doc, &changes, caller).await?;
        } else {
            self.apply(&mut doc, &changes, caller).await?;
        }
        Ok(Some(doc))
    }

    async fn stage(
        &self,
        doc: &mut Liability,
        changes: &LiabilityPatch,
        caller: &CallerContext,
    ) -> Result<(), AppError> {
        let mut combined = staged_patch(doc);
        combined.merge(changes);
        // Setting a field back to its committed value un-stages it.
        combined.retain_different(&doc.fields);
        // Validate the would-be record (e.g. date ordering).
        let mut would_be = doc.fields.clone();
        combined.apply(&mut would_be);
        would_be.check()?;

        doc.staged = if combined.is_empty() {
            None
        } else {
            let stamp = caller.stamp();
            Some(StagedChange {
                action: StagedAction::Update,
                changes: combined.to_staged(),
                created_by: doc
                    .staged
                    .as_ref()
                    .map_or_else(|| stamp.clone(), |s| s.created_by.clone()),
                updated_by: stamp,
            })
        };
        self.save(doc).await
    }

    async fn apply(
        &self,
        doc: &mut Liability,
        changes: &LiabilityPatch,
        caller: &CallerContext,
    ) -> Result<(), AppError> {
        // Re-validate the combined document (e.g. date ordering).
        let mut merged = doc.fields.clone();
        changes.apply(&mut merged);
        merged.check()?;
        doc.fields = merged;
        doc.updated_at = Utc::now();
        doc.updated_by = caller.stamp();

        // The edit wins: drop staged fields it overlaps or that now match the record.
        let mut remaining = staged_patch(doc);
        remaining.remove_keys(changes);
        remaining.retain_different(&doc.fields);

        if let Some(staged) = doc.staged.as_mut()
            && staged.action != StagedAction::Update
        {
            // Pending add/delete: the record changes, the flag stays (client edits to a pending
            // add land here too and just move the latest-edit stamp).
            if caller.caller_type == CallerType::Client {
                staged.updated_by = caller.stamp();
            }
            return self.save(doc).await;
        }
        if !remaining.is_empty() {
            let mut would_be = doc.fields.clone();
            remaining.apply(&mut would_be);
            if would_be.check().is_err() {
                remaining = LiabilityPatch::default(); // leftovers no longer make a valid record
            }
        }
        if let Some(staged) = doc.staged.as_mut() {
            // Prune only: keep the client's original stamps.
            if remaining.is_empty() {
                doc.staged = None;
            } else {
                staged.changes = remaining.to_staged();
            }
        }
        self.save(doc).await
    }

    /// Advisors hard-delete. A client's delete is staged, unless the record is their own
    /// still-pending add (nothing committed to review, so it just goes away).
    pub async fn delete(
        &self,
        client_id: &str,
        liability_id: &str,
        caller: &CallerContext,
    ) -> Result<bool, AppError> {
        let Some(mut doc) = self.get(client_id, liability_id, caller).await? else {
            return Ok(false);
        };
        if caller.caller_type == CallerType::Client
            && doc.staged_action() != Some(StagedAction::Create)
        {
            let stamp: AuditStamp = caller.stamp();
            doc.staged = Some(StagedChange {
                action: StagedAction::Delete,
                changes: Default::default(),
                created_by: stamp.clone(),
                updated_by: stamp,
            });
            self.save(&doc).await?;
        } else if let Some(id) = doc.id {
            self.coll.delete_one(doc! { "_id": id }).await?;
        }
        Ok(true)
    }

    /// Clients get totals over their staged view; advisors over committed data.
    pub async fn summary(
        &self,
        client_id: &str,
        caller: &CallerContext,
    ) -> Result<LiabilitySummary, AppError> {
        let raw = self.find_all(doc! { "client_id": client_id }).await?;
        let is_advisor = caller.caller_type == CallerType::Advisor; // clients get no change info
        let mut pending_by_action: IndexMap<String, u64> = IndexMap::new();
        for d in &raw {
            if let Some(s) = &d.staged {
                *pending_by_action
                    .entry(s.action.as_str().to_owned())
                    .or_default() += 1;
            }
        }
        let fields: Vec<LiabilityFields> = if is_advisor {
            // pending adds are not committed yet
            raw.iter()
                .filter(|d| d.staged_action() != Some(StagedAction::Create))
                .map(|d| d.fields.clone())
                .collect()
        } else {
            // pending deletes are gone, adds are in
            raw.iter()
                .filter(|d| d.staged_action() != Some(StagedAction::Delete))
                .map(effective)
                .collect()
        };

        let mut count_by_type: IndexMap<String, u64> = IndexMap::new();
        let mut balance_by_type: IndexMap<String, Decimal> = IndexMap::new();
        let mut total_balance = Decimal::ZERO;
        let mut annual_payments = Decimal::ZERO;
        for f in &fields {
            let key = f.liability_type.as_str().to_owned();
            *count_by_type.entry(key.clone()).or_default() += 1;
            *balance_by_type.entry(key).or_default() += f.current_balance;
            total_balance += f.current_balance;
            if let (Some(amount), Some(freq)) = (f.payment_amount, f.payment_frequency) {
                annual_payments += amount * Decimal::from(freq.per_year());
            }
        }
        Ok(LiabilitySummary {
            client_id: client_id.to_owned(),
            count: fields.len(),
            total_current_balance: total_balance,
            count_by_type,
            balance_by_type,
            total_annual_payments: annual_payments,
            pending_change_count: if is_advisor {
                pending_by_action.values().sum()
            } else {
                0
            },
            pending_by_action: if is_advisor {
                pending_by_action
            } else {
                IndexMap::new()
            },
        })
    }
}
