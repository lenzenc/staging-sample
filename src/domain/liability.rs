use chrono::{DateTime, NaiveDate, Utc};
use garde::Validate;
use mongodb::bson::oid::ObjectId;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use utoipa::ToSchema;

use super::enums::{
    CallerType, LiabilityType, Ownership, PaymentFrequency, RateType, StagedAction,
};
use crate::error::{AppError, ErrorDetail};

// ---- field rules (the equivalents of Pydantic's `Field(ge=0, max_digits=14, ...)`) ----

/// `digits` counts significant digits, so `1500.50` and `1500.5` are both fine for 2 places.
fn decimal_fits(v: &Decimal, max_digits: u32, places: u32) -> bool {
    let n = v.normalize();
    let digits = n.mantissa().unsigned_abs().to_string().len() as u32;
    n.scale() <= places && digits.max(n.scale()) <= max_digits
}

fn money(v: &Decimal, _: &()) -> garde::Result {
    if v.is_sign_negative() && !v.is_zero() {
        return Err(garde::Error::new("must be greater than or equal to 0"));
    }
    if !decimal_fits(v, 14, 2) {
        return Err(garde::Error::new(
            "must have at most 14 digits and 2 decimal places",
        ));
    }
    Ok(())
}

fn rate(v: &Decimal, _: &()) -> garde::Result {
    if v.is_sign_negative() && !v.is_zero() {
        return Err(garde::Error::new("must be greater than or equal to 0"));
    }
    if *v > Decimal::ONE_HUNDRED {
        return Err(garde::Error::new("must be less than or equal to 100"));
    }
    if !decimal_fits(v, 20, 4) {
        return Err(garde::Error::new("must have at most 4 decimal places"));
    }
    Ok(())
}

fn last4(v: &str, _: &()) -> garde::Result {
    if v.len() == 4 && v.bytes().all(|b| b.is_ascii_digit()) {
        Ok(())
    } else {
        Err(garde::Error::new("must be exactly 4 digits"))
    }
}

/// Fields shared by the stored document and the create body.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Validate, ToSchema)]
pub struct LiabilityFields {
    #[garde(skip)]
    pub liability_type: LiabilityType,
    #[garde(length(chars, min = 1, max = 200))]
    pub name: String,
    #[garde(inner(length(chars, max = 200)))]
    pub lender: Option<String>,
    #[garde(inner(custom(last4)))]
    pub account_last4: Option<String>,
    #[serde(default)]
    #[garde(skip)]
    pub owner: Ownership,
    #[garde(inner(custom(money)))]
    pub original_balance: Option<Decimal>,
    #[garde(custom(money))]
    pub current_balance: Decimal,
    #[garde(inner(custom(rate)))]
    pub interest_rate: Option<Decimal>,
    #[garde(skip)]
    pub rate_type: Option<RateType>,
    #[garde(inner(custom(money)))]
    pub payment_amount: Option<Decimal>,
    #[garde(skip)]
    pub payment_frequency: Option<PaymentFrequency>,
    #[garde(skip)]
    pub origination_date: Option<NaiveDate>,
    #[garde(skip)]
    pub maturity_date: Option<NaiveDate>,
    #[serde(default)]
    #[garde(skip)]
    pub is_tax_deductible: bool,
    #[garde(inner(length(chars, max = 2000)))]
    pub notes: Option<String>,
}

impl LiabilityFields {
    /// Field rules plus the cross-field date check. Errors are FastAPI-style 422 details.
    pub fn check(&self) -> Result<(), AppError> {
        let mut errors: Vec<ErrorDetail> = match self.validate() {
            Ok(()) => Vec::new(),
            Err(report) => report
                .iter()
                .map(|(path, err)| {
                    let loc = path.to_string();
                    let loc: Vec<&str> = std::iter::once("body").chain(loc.split('.')).collect();
                    ErrorDetail::new(&loc, err.message())
                })
                .collect(),
        };
        if let (Some(start), Some(end)) = (self.origination_date, self.maturity_date)
            && end < start
        {
            errors.push(ErrorDetail::new(
                &["body"],
                "maturity_date must be on or after origination_date",
            ));
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(AppError::Validation(errors))
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct AuditStamp {
    pub user_id: String,
    pub caller_type: CallerType,
    pub at: DateTime<Utc>,
}

impl AuditStamp {
    pub fn now(user_id: impl Into<String>, caller_type: CallerType) -> Self {
        Self {
            user_id: user_id.into(),
            caller_type,
            at: Utc::now(),
        }
    }
}

/// A client's proposed add, edit or delete, held until an advisor reviews it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StagedChange {
    #[serde(default)]
    pub action: StagedAction,
    /// For updates: sparse field -> proposed value in JSON form (decimals as strings, dates as
    /// ISO strings). Empty for create and delete.
    #[serde(default)]
    pub changes: Map<String, Value>,
    pub created_by: AuditStamp,
    pub updated_by: AuditStamp,
}

/// The stored liability. `id` is carried separately from the serialized body.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Liability {
    #[serde(skip)]
    pub id: Option<ObjectId>,
    pub client_id: String,
    #[serde(flatten)]
    pub fields: LiabilityFields,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: AuditStamp,
    pub updated_by: AuditStamp,
    pub staged: Option<StagedChange>,
}

impl Liability {
    pub fn staged_action(&self) -> Option<StagedAction> {
        self.staged.as_ref().map(|s| s.action)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn base() -> serde_json::Value {
        json!({"liability_type": "mortgage", "name": "Home", "current_balance": "250000.00"})
    }

    fn parse(patch: serde_json::Value) -> LiabilityFields {
        let mut v = base();
        v.as_object_mut()
            .unwrap()
            .extend(patch.as_object().unwrap().clone());
        serde_json::from_value(v).unwrap()
    }

    #[test]
    fn accepts_a_minimal_record() {
        assert!(parse(json!({})).check().is_ok());
    }

    #[test]
    fn rejects_bad_values() {
        for bad in [
            json!({"current_balance": "-1"}),
            json!({"current_balance": "1.234"}),
            json!({"account_last4": "12"}),
            json!({"interest_rate": "150"}),
            json!({"name": ""}),
            json!({"origination_date": "2020-01-01", "maturity_date": "2019-01-01"}),
        ] {
            assert!(parse(bad.clone()).check().is_err(), "{bad}");
        }
    }

    #[test]
    fn trailing_zeros_do_not_count_as_places() {
        assert!(
            parse(json!({"current_balance": "1500.500"}))
                .check()
                .is_ok()
        );
    }

    #[test]
    fn null_current_balance_is_a_parse_error() {
        let mut v = base();
        v["current_balance"] = json!(null);
        assert!(serde_json::from_value::<LiabilityFields>(v).is_err());
    }
}
