//! Maps a [`Liability`] to and from its MongoDB document.
//!
//! The API types serialize money and dates as strings. In MongoDB the shape matches the Python
//! app: money as Decimal128, `date` fields and timestamps as BSON datetimes. Only these known
//! paths are converted; everything else (including `staged.changes`) stays as plain JSON.

use std::str::FromStr;

use chrono::{DateTime, NaiveDate, SecondsFormat, Utc};
use mongodb::bson::{self, Bson, Document, oid::ObjectId};
use rust_decimal::Decimal;

use crate::domain::liability::Liability;
use crate::error::AppError;

const MONEY: [&str; 4] = [
    "original_balance",
    "current_balance",
    "interest_rate",
    "payment_amount",
];
const DATES: [&str; 2] = ["origination_date", "maturity_date"];
const TIMESTAMPS: [&str; 2] = ["created_at", "updated_at"];
const STAMPS: [&str; 2] = ["created_by", "updated_by"];

fn codec_err(msg: impl std::fmt::Display) -> AppError {
    AppError::Codec(msg.to_string())
}

pub fn encode(liability: &Liability) -> Result<Document, AppError> {
    let mut doc = bson::serialize_to_document(liability).map_err(codec_err)?;
    for key in MONEY {
        if let Some(Bson::String(s)) = doc.get(key) {
            let d = bson::Decimal128::from_str(s).map_err(codec_err)?;
            doc.insert(key, d);
        }
    }
    for key in DATES {
        if let Some(Bson::String(s)) = doc.get(key) {
            let date = NaiveDate::from_str(s).map_err(codec_err)?;
            let midnight = date
                .and_hms_opt(0, 0, 0)
                .expect("midnight is valid")
                .and_utc();
            doc.insert(key, bson::DateTime::from_chrono(midnight));
        }
    }
    for key in TIMESTAMPS {
        string_to_datetime(&mut doc, key)?;
    }
    for key in STAMPS {
        if let Ok(stamp) = doc.get_document_mut(key) {
            string_to_datetime(stamp, "at")?;
        }
    }
    if let Ok(staged) = doc.get_document_mut("staged") {
        for key in STAMPS {
            if let Ok(stamp) = staged.get_document_mut(key) {
                string_to_datetime(stamp, "at")?;
            }
        }
    }
    Ok(doc)
}

pub fn decode(mut doc: Document) -> Result<Liability, AppError> {
    let id: Option<ObjectId> = doc.get_object_id("_id").ok();
    doc.remove("_id");
    for key in MONEY {
        if let Some(Bson::Decimal128(d)) = doc.get(key) {
            let s = d.to_string();
            let parsed = Decimal::from_str(&s)
                .or_else(|_| Decimal::from_scientific(&s))
                .map_err(codec_err)?;
            doc.insert(key, parsed.to_string());
        }
    }
    for key in DATES {
        if let Some(Bson::DateTime(dt)) = doc.get(key) {
            let date = dt.to_chrono().date_naive();
            doc.insert(key, date.to_string());
        }
    }
    for key in TIMESTAMPS {
        datetime_to_string(&mut doc, key);
    }
    for key in STAMPS {
        if let Ok(stamp) = doc.get_document_mut(key) {
            datetime_to_string(stamp, "at");
        }
    }
    if let Ok(staged) = doc.get_document_mut("staged") {
        for key in STAMPS {
            if let Ok(stamp) = staged.get_document_mut(key) {
                datetime_to_string(stamp, "at");
            }
        }
    }
    let mut liability: Liability = bson::deserialize_from_document(doc).map_err(codec_err)?;
    liability.id = id;
    Ok(liability)
}

fn string_to_datetime(doc: &mut Document, key: &str) -> Result<(), AppError> {
    if let Some(Bson::String(s)) = doc.get(key) {
        let dt = DateTime::parse_from_rfc3339(s)
            .map_err(codec_err)?
            .with_timezone(&Utc);
        doc.insert(key, bson::DateTime::from_chrono(dt));
    }
    Ok(())
}

fn datetime_to_string(doc: &mut Document, key: &str) {
    if let Some(Bson::DateTime(dt)) = doc.get(key) {
        let s = dt.to_chrono().to_rfc3339_opts(SecondsFormat::AutoSi, true);
        doc.insert(key, s);
    }
}
