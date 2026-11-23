use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

macro_rules! wire_enum {
    ($(#[$m:meta])* $name:ident { $($(#[$vm:meta])* $variant:ident => $wire:literal),+ $(,)? }) => {
        $(#[$m])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
        pub enum $name {
            $($(#[$vm])* #[serde(rename = $wire)] $variant),+
        }

        impl $name {
            /// The value used on the wire and in MongoDB.
            pub fn as_str(self) -> &'static str {
                match self { $(Self::$variant => $wire),+ }
            }
        }
    };
}

wire_enum!(CallerType { Advisor => "advisor", Client => "client" });

wire_enum!(LiabilityType {
    Mortgage => "mortgage",
    HomeEquityLoan => "home_equity_loan",
    Heloc => "heloc",
    AutoLoan => "auto_loan",
    StudentLoan => "student_loan",
    CreditCard => "credit_card",
    PersonalLoan => "personal_loan",
    MedicalDebt => "medical_debt",
    BusinessLoan => "business_loan",
    TaxLiability => "tax_liability",
    Other => "other",
});

wire_enum!(
    #[derive(Default)]
    Ownership {
        #[default]
        Client => "client",
        Spouse => "spouse",
        Joint => "joint",
    }
);

wire_enum!(RateType { Fixed => "fixed", Variable => "variable" });

wire_enum!(PaymentFrequency {
    Weekly => "weekly",
    Biweekly => "biweekly",
    Monthly => "monthly",
    Quarterly => "quarterly",
    Annually => "annually",
});

wire_enum!(
    #[derive(Default)]
    StagedAction {
        Create => "create",
        #[default]
        Update => "update",
        Delete => "delete",
    }
);

impl PaymentFrequency {
    /// Payments per year, used to normalize payments in the summary.
    pub fn per_year(self) -> u32 {
        match self {
            Self::Weekly => 52,
            Self::Biweekly => 26,
            Self::Monthly => 12,
            Self::Quarterly => 4,
            Self::Annually => 1,
        }
    }
}
