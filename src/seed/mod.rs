//! Sample client scenarios, loaded from `seed/scenarios.json`.
//!
//! Each scenario is a client ID with a title, description and liabilities. A liability may carry
//! `by: "client"` (entered by the client rather than an advisor), `staged` (a sparse edit) and
//! `staged_action` (`create` or `delete`).

use std::sync::OnceLock;

use chrono::{Duration, Utc};
use mongodb::bson::doc;
use serde::Deserialize;
use serde_json::{Map, Value};

use crate::codec;
use crate::db::Store;
use crate::domain::enums::{CallerType, StagedAction};
use crate::domain::liability::{AuditStamp, Liability, LiabilityFields, StagedChange};
use crate::dto::{ScenarioCounts, ScenarioInfo};
use crate::error::AppError;

const SCENARIOS_JSON: &str = include_str!("../../seed/scenarios.json");

#[derive(Debug, Deserialize)]
struct ScenarioFile {
    scenarios: Vec<Scenario>,
}

#[derive(Debug, Deserialize)]
pub struct Scenario {
    pub client_id: String,
    pub title: String,
    pub description: String,
    liabilities: Vec<SeedLiability>,
}

#[derive(Debug, Deserialize)]
struct SeedLiability {
    by: Option<String>,
    staged: Option<Map<String, Value>>,
    staged_action: Option<StagedAction>,
    #[serde(flatten)]
    fields: LiabilityFields,
}

impl SeedLiability {
    fn is_edit(&self) -> bool {
        self.staged.is_some() && self.staged_action.is_none()
    }
}

pub fn scenarios() -> &'static [Scenario] {
    static CELL: OnceLock<Vec<Scenario>> = OnceLock::new();
    CELL.get_or_init(|| {
        serde_json::from_str::<ScenarioFile>(SCENARIOS_JSON)
            .expect("seed/scenarios.json is valid")
            .scenarios
    })
}

impl Scenario {
    /// Counts of liabilities by staged kind.
    pub fn counts(&self) -> ScenarioCounts {
        let count = |action| {
            self.liabilities
                .iter()
                .filter(|l| l.staged_action == Some(action))
                .count()
        };
        let edits = self.liabilities.iter().filter(|l| l.is_edit()).count();
        let (adds, deletes) = (count(StagedAction::Create), count(StagedAction::Delete));
        ScenarioCounts {
            liabilities: self.liabilities.len(),
            unchanged: self.liabilities.len() - edits - adds - deletes,
            edits,
            adds,
            deletes,
        }
    }

    pub fn info(&self) -> ScenarioInfo {
        ScenarioInfo {
            client_id: self.client_id.clone(),
            title: self.title.clone(),
            description: self.description.clone(),
            counts: self.counts(),
        }
    }
}

/// Replace the seeded clients' liabilities with the sample data. Returns how many were inserted.
pub async fn seed(store: &Store) -> Result<usize, AppError> {
    let ids: Vec<&str> = scenarios().iter().map(|s| s.client_id.as_str()).collect();
    store
        .coll
        .delete_many(doc! { "client_id": { "$in": ids } })
        .await?;

    let now = Utc::now();
    let mut inserted = 0;
    for scenario in scenarios() {
        let client_user = format!("cli-{}", scenario.client_id);
        for (i, item) in scenario.liabilities.iter().enumerate() {
            let stamp = if item.by.as_deref() == Some("client") {
                AuditStamp {
                    user_id: client_user.clone(),
                    caller_type: CallerType::Client,
                    at: now,
                }
            } else {
                AuditStamp {
                    user_id: "adv-1".into(),
                    caller_type: CallerType::Advisor,
                    at: now,
                }
            };
            let at = now - Duration::days(30 - i as i64);
            let stamp = AuditStamp { at, ..stamp };

            let action = item.staged_action.unwrap_or(StagedAction::Update);
            let staged = (item.staged.is_some() || action != StagedAction::Update).then(|| {
                // the client's pending change: first made 3 days ago, last 2 hours ago
                let by_client = |at| AuditStamp {
                    user_id: client_user.clone(),
                    caller_type: CallerType::Client,
                    at,
                };
                StagedChange {
                    action,
                    changes: item.staged.clone().unwrap_or_default(),
                    created_by: by_client(now - Duration::days(3)),
                    updated_by: by_client(now - Duration::hours(2)),
                }
            });
            let liability = Liability {
                id: None,
                client_id: scenario.client_id.clone(),
                fields: item.fields.clone(),
                created_at: at,
                updated_at: at,
                created_by: stamp.clone(),
                updated_by: stamp,
                staged,
            };
            store.coll.insert_one(codec::encode(&liability)?).await?;
            inserted += 1;
        }
    }
    Ok(inserted)
}
