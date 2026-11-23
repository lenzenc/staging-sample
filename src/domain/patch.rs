use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::{Map, Value};
use utoipa::ToSchema;

use super::enums::{LiabilityType, Ownership, PaymentFrequency, RateType};
use super::liability::LiabilityFields;
use crate::error::{AppError, ErrorDetail};

/// A PATCH value that tells "not sent" apart from "sent as null" (Pydantic's `model_fields_set`).
#[derive(Debug, Clone, Default, PartialEq)]
pub enum Patch<T> {
    #[default]
    Missing,
    Null,
    Value(T),
}

impl<T> Patch<T> {
    pub fn is_missing(&self) -> bool {
        matches!(self, Self::Missing)
    }
}

impl<'de, T: Deserialize<'de>> Deserialize<'de> for Patch<T> {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        Ok(match Option::<T>::deserialize(d)? {
            Some(v) => Self::Value(v),
            None => Self::Null,
        })
    }
}

impl<T: Serialize> Serialize for Patch<T> {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Value(v) => v.serialize(s),
            Self::Null | Self::Missing => s.serialize_none(),
        }
    }
}

/// One field's change as `(name, current, proposed)` in JSON form.
pub type FieldDiff = (&'static str, Value, Value);

macro_rules! liability_patch {
    (
        required: [$($req:ident: $rty:ty),+ $(,)?],
        optional: [$($opt:ident: $oty:ty),+ $(,)?] $(,)?
    ) => {
        /// PATCH body, and the shape of a client's staged edits: every field optional,
        /// only fields sent are changed. Optional fields may be set to `null` to clear them.
        #[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, ToSchema)]
        pub struct LiabilityPatch {
            $(#[serde(default, skip_serializing_if = "Patch::is_missing")] #[schema(value_type = Option<$rty>)] pub $req: Patch<$rty>,)+
            $(#[serde(default, skip_serializing_if = "Patch::is_missing")] #[schema(value_type = Option<$oty>)] pub $opt: Patch<$oty>,)+
        }

        impl LiabilityPatch {
            /// Required fields that were sent as an explicit null.
            pub fn null_required(&self) -> Vec<&'static str> {
                let mut out = Vec::new();
                $(if matches!(self.$req, Patch::Null) { out.push(stringify!($req)); })+
                out
            }

            /// Overlay the sent fields onto `f`.
            pub fn apply(&self, f: &mut LiabilityFields) {
                $(if let Patch::Value(v) = &self.$req { f.$req = v.clone(); })+
                $(match &self.$opt {
                    Patch::Missing => {}
                    Patch::Null => f.$opt = None,
                    Patch::Value(v) => f.$opt = Some(v.clone()),
                })+
            }

            /// Take `other`'s sent fields over this patch's (newer edit wins).
            pub fn merge(&mut self, other: &Self) {
                $(if !other.$req.is_missing() { self.$req = other.$req.clone(); })+
                $(if !other.$opt.is_missing() { self.$opt = other.$opt.clone(); })+
            }

            /// Drop the fields that `other` also sets.
            pub fn remove_keys(&mut self, other: &Self) {
                $(if !other.$req.is_missing() { self.$req = Patch::Missing; })+
                $(if !other.$opt.is_missing() { self.$opt = Patch::Missing; })+
            }

            /// Drop fields whose value already equals the one in `base`.
            pub fn retain_different(&mut self, base: &LiabilityFields) {
                $(if matches!(&self.$req, Patch::Value(v) if *v == base.$req) {
                    self.$req = Patch::Missing;
                })+
                $(match &self.$opt {
                    Patch::Null if base.$opt.is_none() => self.$opt = Patch::Missing,
                    Patch::Value(v) if base.$opt.as_ref() == Some(v) => self.$opt = Patch::Missing,
                    _ => {}
                })+
            }

            pub fn is_empty(&self) -> bool {
                true $(&& self.$req.is_missing())+ $(&& self.$opt.is_missing())+
            }

            /// Each staged field as `(name, current, proposed)` against `base`.
            pub fn diff(&self, base: &LiabilityFields) -> Vec<FieldDiff> {
                let mut out = Vec::new();
                $(if let Patch::Value(v) = &self.$req {
                    out.push((stringify!($req), json(&base.$req), json(v)));
                })+
                $(match &self.$opt {
                    Patch::Missing => {}
                    Patch::Null => out.push((stringify!($opt), json(&base.$opt), Value::Null)),
                    Patch::Value(v) => out.push((stringify!($opt), json(&base.$opt), json(v))),
                })+
                out
            }
        }
    };
}

fn json<T: Serialize>(v: &T) -> Value {
    serde_json::to_value(v).unwrap_or(Value::Null)
}

liability_patch! {
    required: [
        liability_type: LiabilityType,
        name: String,
        owner: Ownership,
        current_balance: Decimal,
        is_tax_deductible: bool,
    ],
    optional: [
        lender: String,
        account_last4: String,
        original_balance: Decimal,
        interest_rate: Decimal,
        rate_type: RateType,
        payment_amount: Decimal,
        payment_frequency: PaymentFrequency,
        origination_date: NaiveDate,
        maturity_date: NaiveDate,
        notes: String,
    ],
}

impl LiabilityPatch {
    /// Reject nulls on required fields, then check each sent field against the same rules as a
    /// full record by applying the patch to a known-good placeholder.
    pub fn check(&self) -> Result<(), AppError> {
        let nulls = self.null_required();
        if !nulls.is_empty() {
            return Err(AppError::Validation(
                nulls
                    .into_iter()
                    .map(|f| ErrorDetail::new(&["body", f], format!("{f} cannot be null")))
                    .collect(),
            ));
        }
        let mut probe = LiabilityFields {
            liability_type: LiabilityType::Other,
            name: "placeholder".into(),
            lender: None,
            account_last4: None,
            owner: Ownership::Client,
            original_balance: None,
            current_balance: Decimal::ZERO,
            interest_rate: None,
            rate_type: None,
            payment_amount: None,
            payment_frequency: None,
            origination_date: None,
            maturity_date: None,
            is_tax_deductible: false,
            notes: None,
        };
        self.apply(&mut probe);
        probe.check()
    }

    /// Typed view of the JSON stored in `staged.changes`.
    pub fn from_staged(changes: &Map<String, Value>) -> Self {
        serde_json::from_value(Value::Object(changes.clone())).unwrap_or_default()
    }

    /// The JSON form stored in `staged.changes`.
    pub fn to_staged(&self) -> Map<String, Value> {
        match serde_json::to_value(self) {
            Ok(Value::Object(m)) => m,
            _ => Map::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn patch(v: serde_json::Value) -> LiabilityPatch {
        serde_json::from_value(v).unwrap()
    }

    #[test]
    fn tells_missing_from_null() {
        let p = patch(json!({"notes": null, "lender": "Chase"}));
        assert_eq!(p.notes, Patch::Null);
        assert_eq!(p.lender, Patch::Value("Chase".into()));
        assert!(p.interest_rate.is_missing());
    }

    #[test]
    fn staged_json_round_trips_with_explicit_null() {
        let p = patch(json!({"notes": null, "current_balance": "900.00"}));
        let stored = p.to_staged();
        assert_eq!(stored["notes"], Value::Null);
        assert_eq!(stored["current_balance"], json!("900.00"));
        assert!(!stored.contains_key("lender"));
        assert_eq!(LiabilityPatch::from_staged(&stored), p);
    }

    #[test]
    fn null_on_required_field_is_rejected() {
        assert!(patch(json!({"current_balance": null})).check().is_err());
        assert!(patch(json!({"notes": null})).check().is_ok());
    }

    #[test]
    fn bad_field_value_is_rejected() {
        assert!(patch(json!({"account_last4": "12"})).check().is_err());
        assert!(
            patch(json!({"origination_date": "2020-01-01", "maturity_date": "2019-01-01"}))
                .check()
                .is_err()
        );
    }
}
