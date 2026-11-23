use chrono::{DateTime, Utc};
use indexmap::IndexMap;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use utoipa::{IntoParams, ToSchema};

use crate::domain::enums::{LiabilityType, StagedAction};
use crate::domain::liability::{AuditStamp, LiabilityFields};

/// A liability as a client sees it: committed data merged with their staged changes, and no
/// change information at all.
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ClientLiabilityView {
    pub id: String,
    pub client_id: String,
    #[serde(flatten)]
    pub fields: LiabilityFields,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: AuditStamp,
    pub updated_by: AuditStamp,
}

/// The advisor's view: committed data plus what the client has staged (`null` when nothing).
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct AdvisorLiabilityView {
    #[serde(flatten)]
    pub base: ClientLiabilityView,
    pub pending_change: Option<PendingChange>,
}

/// Serialized as whichever variant applies, so a client payload has no `pending_change` key.
#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(untagged)]
pub enum LiabilityView {
    Client(ClientLiabilityView),
    Advisor(AdvisorLiabilityView),
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct FieldChange {
    #[schema(value_type = Object)]
    pub current: Value,
    #[schema(value_type = Object)]
    pub proposed: Value,
}

/// What a client has staged for a liability, as current -> proposed per field.
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct PendingChange {
    pub action: StagedAction,
    #[schema(value_type = HashMap<String, FieldChange>)]
    pub changes: IndexMap<String, FieldChange>,
    pub created_by: AuditStamp,
    pub updated_by: AuditStamp,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct LiabilityList {
    pub items: Vec<LiabilityView>,
    pub total: u64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct LiabilitySummary {
    pub client_id: String,
    pub count: usize,
    pub total_current_balance: Decimal,
    #[schema(value_type = HashMap<String, u64>)]
    pub count_by_type: IndexMap<String, u64>,
    #[schema(value_type = HashMap<String, String>)]
    pub balance_by_type: IndexMap<String, Decimal>,
    pub total_annual_payments: Decimal,
    pub pending_change_count: u64,
    #[schema(value_type = HashMap<String, u64>)]
    pub pending_by_action: IndexMap<String, u64>,
}

/// Query parameters for the list endpoint.
#[derive(Debug, Deserialize, IntoParams)]
pub struct ListParams {
    pub liability_type: Option<LiabilityType>,
    pub skip: Option<u64>,
    pub limit: Option<u64>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ScenarioCounts {
    pub liabilities: usize,
    pub unchanged: usize,
    pub edits: usize,
    pub adds: usize,
    pub deletes: usize,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ScenarioInfo {
    pub client_id: String,
    pub title: String,
    pub description: String,
    pub counts: ScenarioCounts,
}
