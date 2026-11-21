# Advisor vs client views, per seed use case

Each section shows the full `GET /api/v1/clients/{client_id}/liabilities` response for one seed use (`make seed`), once as the advisor and once as the client. Captured from the running API.

| Caller | Headers |
|---|---|
| Advisor | `X-Caller-Type: advisor`, `X-User-Id: adv-1` |
| Client | `X-Caller-Type: client`, `X-User-Id: cli-<client_id>` |

IDs and timestamps are regenerated on every `make seed`, so yours will differ.

## What differs between the views

- Only the advisor sees `pending_change`. Client responses have no such key.
- The client sees staged edits merged over the committed values.
- A staged add looks like a normal record to the client; the advisor sees it flagged `create`.
- A staged delete is hidden from the client; the advisor still sees the record flagged `delete`.

## Use cases

- [Mixed: edits + add — `client-young-professional`](#mixed-edits--add--client-young-professional)
- [All three kinds — `client-family-homeowners`](#all-three-kinds--client-family-homeowners)
- [Edit + delete — `client-pre-retiree`](#edit--delete--client-pre-retiree)
- [Edit + add, cleared field — `client-small-business-owner`](#edit--add-cleared-field--client-small-business-owner)
- [Edit + delete, high balances — `client-medical-professional`](#edit--delete-high-balances--client-medical-professional)
- [No staged changes — `client-no-pending`](#no-staged-changes--client-no-pending)
- [Staged edits only — `client-edits-only`](#staged-edits-only--client-edits-only)
- [Staged adds only, new client — `client-adds-only`](#staged-adds-only-new-client--client-adds-only)
- [Single staged add — `client-single-add`](#single-staged-add--client-single-add)
- [Staged deletes only — `client-deletes-only`](#staged-deletes-only--client-deletes-only)
- [No liabilities — `client-empty`](#no-liabilities--client-empty)

## Mixed: edits + add — `client-young-professional`

Late 20s renter: student loans, car loan, card balance

Liabilities: 4 · unchanged: 0 · staged edits: 3 · adds: 1 · deletes: 0

**What to notice**

- Advisor list has **4** item(s); client list has **4**.
- **Affirm - laptop** (staged add): the client sees it as a normal record; the advisor sees it flagged `create` (and the advisor's summary leaves it out until accepted).
- **Rewards Visa** (staged edit): advisor sees committed values plus `pending_change` (`current_balance` 3120.40 → 2400.00; `payment_amount` 150.00 → 400.00); the client sees the proposed values merged in and no `pending_change` key.
- **2022 Honda Civic** (staged edit): advisor sees committed values plus `pending_change` (`current_balance` 14200.00 → 13100.00); the client sees the proposed values merged in and no `pending_change` key.
- **Federal student loans** (staged edit): advisor sees committed values plus `pending_change` (`current_balance` 31850.25 → 30100.00; `interest_rate` 5.05 → 4.53); the client sees the proposed values merged in and no `pending_change` key.

<table>
<tr><th>Advisor GET</th><th>Client GET</th></tr>
<tr>
<td valign="top">

```json
{
  "items": [
    {
      "liability_type": "personal_loan",
      "name": "Affirm - laptop",
      "lender": "Affirm",
      "account_last4": null,
      "owner": "client",
      "original_balance": "1380",
      "current_balance": "1150.00",
      "interest_rate": "15.0",
      "rate_type": "fixed",
      "payment_amount": "115.00",
      "payment_frequency": "monthly",
      "origination_date": null,
      "maturity_date": null,
      "is_tax_deductible": false,
      "notes": null,
      "id": "6b00ef9a7c1b5f710ea8c4ff",
      "client_id": "client-young-professional",
      "created_at": "2026-10-25T01:10:50.228000Z",
      "updated_at": "2026-10-25T01:10:50.228000Z",
      "created_by": {
        "user_id": "cli-client-young-professional",
        "caller_type": "client",
        "at": "2026-10-25T01:10:50.228000Z"
      },
      "updated_by": {
        "user_id": "cli-client-young-professional",
        "caller_type": "client",
        "at": "2026-10-25T01:10:50.228000Z"
      },
      "pending_change": {
        "action": "create",
        "changes": {},
        "created_by": {
          "user_id": "cli-client-young-professional",
          "caller_type": "client",
          "at": "2026-11-18T01:10:50.228000Z"
        },
        "updated_by": {
          "user_id": "cli-client-young-professional",
          "caller_type": "client",
          "at": "2026-11-20T23:10:50.228000Z"
        }
      }
    },
    {
      "liability_type": "credit_card",
      "name": "Rewards Visa",
      "lender": "Chase",
      "account_last4": "9012",
      "owner": "client",
      "original_balance": null,
      "current_balance": "3120.40",
      "interest_rate": "22.99",
      "rate_type": "variable",
      "payment_amount": "150.00",
      "payment_frequency": "monthly",
      "origination_date": null,
      "maturity_date": null,
      "is_tax_deductible": false,
      "notes": "Carries a balance most months; candidate for payoff plan.",
      "id": "6b00ef9a7c1b5f710ea8c4fe",
      "client_id": "client-young-professional",
      "created_at": "2026-10-24T01:10:50.228000Z",
      "updated_at": "2026-10-24T01:10:50.228000Z",
      "created_by": {
        "user_id": "cli-client-young-professional",
        "caller_type": "client",
        "at": "2026-10-24T01:10:50.228000Z"
      },
      "updated_by": {
        "user_id": "cli-client-young-professional",
        "caller_type": "client",
        "at": "2026-10-24T01:10:50.228000Z"
      },
      "pending_change": {
        "action": "update",
        "changes": {
          "current_balance": {
            "current": "3120.40",
            "proposed": "2400.00"
          },
          "payment_amount": {
            "current": "150.00",
            "proposed": "400.00"
          }
        },
        "created_by": {
          "user_id": "cli-client-young-professional",
          "caller_type": "client",
          "at": "2026-11-18T01:10:50.228000Z"
        },
        "updated_by": {
          "user_id": "cli-client-young-professional",
          "caller_type": "client",
          "at": "2026-11-20T23:10:50.228000Z"
        }
      }
    },
    {
      "liability_type": "auto_loan",
      "name": "2022 Honda Civic",
      "lender": "Capital One Auto",
      "account_last4": "4471",
      "owner": "client",
      "original_balance": "24500",
      "current_balance": "14200.00",
      "interest_rate": "4.9",
      "rate_type": "fixed",
      "payment_amount": "465.00",
      "payment_frequency": "monthly",
      "origination_date": "2022-03-15",
      "maturity_date": "2027-03-15",
      "is_tax_deductible": false,
      "notes": null,
      "id": "6b00ef9a7c1b5f710ea8c4fd",
      "client_id": "client-young-professional",
      "created_at": "2026-10-23T01:10:50.228000Z",
      "updated_at": "2026-10-23T01:10:50.228000Z",
      "created_by": {
        "user_id": "cli-client-young-professional",
        "caller_type": "client",
        "at": "2026-10-23T01:10:50.228000Z"
      },
      "updated_by": {
        "user_id": "cli-client-young-professional",
        "caller_type": "client",
        "at": "2026-10-23T01:10:50.228000Z"
      },
      "pending_change": {
        "action": "update",
        "changes": {
          "current_balance": {
            "current": "14200.00",
            "proposed": "13100.00"
          }
        },
        "created_by": {
          "user_id": "cli-client-young-professional",
          "caller_type": "client",
          "at": "2026-11-18T01:10:50.228000Z"
        },
        "updated_by": {
          "user_id": "cli-client-young-professional",
          "caller_type": "client",
          "at": "2026-11-20T23:10:50.228000Z"
        }
      }
    },
    {
      "liability_type": "student_loan",
      "name": "Federal student loans",
      "lender": "Dept. of Education",
      "account_last4": null,
      "owner": "client",
      "original_balance": "42000",
      "current_balance": "31850.25",
      "interest_rate": "5.05",
      "rate_type": "fixed",
      "payment_amount": "385.00",
      "payment_frequency": "monthly",
      "origination_date": "2018-09-01",
      "maturity_date": "2033-09-01",
      "is_tax_deductible": true,
      "notes": null,
      "id": "6b00ef9a7c1b5f710ea8c4fc",
      "client_id": "client-young-professional",
      "created_at": "2026-10-22T01:10:50.228000Z",
      "updated_at": "2026-10-22T01:10:50.228000Z",
      "created_by": {
        "user_id": "adv-1",
        "caller_type": "advisor",
        "at": "2026-10-22T01:10:50.228000Z"
      },
      "updated_by": {
        "user_id": "adv-1",
        "caller_type": "advisor",
        "at": "2026-10-22T01:10:50.228000Z"
      },
      "pending_change": {
        "action": "update",
        "changes": {
          "current_balance": {
            "current": "31850.25",
            "proposed": "30100.00"
          },
          "interest_rate": {
            "current": "5.05",
            "proposed": "4.53"
          }
        },
        "created_by": {
          "user_id": "cli-client-young-professional",
          "caller_type": "client",
          "at": "2026-11-18T01:10:50.228000Z"
        },
        "updated_by": {
          "user_id": "cli-client-young-professional",
          "caller_type": "client",
          "at": "2026-11-20T23:10:50.228000Z"
        }
      }
    }
  ],
  "total": 4
}
```

</td>
<td valign="top">

```json
{
  "items": [
    {
      "liability_type": "personal_loan",
      "name": "Affirm - laptop",
      "lender": "Affirm",
      "account_last4": null,
      "owner": "client",
      "original_balance": "1380",
      "current_balance": "1150.00",
      "interest_rate": "15.0",
      "rate_type": "fixed",
      "payment_amount": "115.00",
      "payment_frequency": "monthly",
      "origination_date": null,
      "maturity_date": null,
      "is_tax_deductible": false,
      "notes": null,
      "id": "6b00ef9a7c1b5f710ea8c4ff",
      "client_id": "client-young-professional",
      "created_at": "2026-10-25T01:10:50.228000Z",
      "updated_at": "2026-10-25T01:10:50.228000Z",
      "created_by": {
        "user_id": "cli-client-young-professional",
        "caller_type": "client",
        "at": "2026-10-25T01:10:50.228000Z"
      },
      "updated_by": {
        "user_id": "cli-client-young-professional",
        "caller_type": "client",
        "at": "2026-10-25T01:10:50.228000Z"
      }
    },
    {
      "liability_type": "credit_card",
      "name": "Rewards Visa",
      "lender": "Chase",
      "account_last4": "9012",
      "owner": "client",
      "original_balance": null,
      "current_balance": "2400.00",
      "interest_rate": "22.99",
      "rate_type": "variable",
      "payment_amount": "400.00",
      "payment_frequency": "monthly",
      "origination_date": null,
      "maturity_date": null,
      "is_tax_deductible": false,
      "notes": "Carries a balance most months; candidate for payoff plan.",
      "id": "6b00ef9a7c1b5f710ea8c4fe",
      "client_id": "client-young-professional",
      "created_at": "2026-10-24T01:10:50.228000Z",
      "updated_at": "2026-10-24T01:10:50.228000Z",
      "created_by": {
        "user_id": "cli-client-young-professional",
        "caller_type": "client",
        "at": "2026-10-24T01:10:50.228000Z"
      },
      "updated_by": {
        "user_id": "cli-client-young-professional",
        "caller_type": "client",
        "at": "2026-10-24T01:10:50.228000Z"
      }
    },
    {
      "liability_type": "auto_loan",
      "name": "2022 Honda Civic",
      "lender": "Capital One Auto",
      "account_last4": "4471",
      "owner": "client",
      "original_balance": "24500",
      "current_balance": "13100.00",
      "interest_rate": "4.9",
      "rate_type": "fixed",
      "payment_amount": "465.00",
      "payment_frequency": "monthly",
      "origination_date": "2022-03-15",
      "maturity_date": "2027-03-15",
      "is_tax_deductible": false,
      "notes": null,
      "id": "6b00ef9a7c1b5f710ea8c4fd",
      "client_id": "client-young-professional",
      "created_at": "2026-10-23T01:10:50.228000Z",
      "updated_at": "2026-10-23T01:10:50.228000Z",
      "created_by": {
        "user_id": "cli-client-young-professional",
        "caller_type": "client",
        "at": "2026-10-23T01:10:50.228000Z"
      },
      "updated_by": {
        "user_id": "cli-client-young-professional",
        "caller_type": "client",
        "at": "2026-10-23T01:10:50.228000Z"
      }
    },
    {
      "liability_type": "student_loan",
      "name": "Federal student loans",
      "lender": "Dept. of Education",
      "account_last4": null,
      "owner": "client",
      "original_balance": "42000",
      "current_balance": "30100.00",
      "interest_rate": "4.53",
      "rate_type": "fixed",
      "payment_amount": "385.00",
      "payment_frequency": "monthly",
      "origination_date": "2018-09-01",
      "maturity_date": "2033-09-01",
      "is_tax_deductible": true,
      "notes": null,
      "id": "6b00ef9a7c1b5f710ea8c4fc",
      "client_id": "client-young-professional",
      "created_at": "2026-10-22T01:10:50.228000Z",
      "updated_at": "2026-10-22T01:10:50.228000Z",
      "created_by": {
        "user_id": "adv-1",
        "caller_type": "advisor",
        "at": "2026-10-22T01:10:50.228000Z"
      },
      "updated_by": {
        "user_id": "adv-1",
        "caller_type": "advisor",
        "at": "2026-10-22T01:10:50.228000Z"
      }
    }
  ],
  "total": 4
}
```

</td>
</tr>
</table>

## All three kinds — `client-family-homeowners`

Married couple, two kids: mortgage, HELOC, minivan loan, closed card, ortho plan

Liabilities: 5 · unchanged: 1 · staged edits: 2 · adds: 1 · deletes: 1

**What to notice**

- Advisor list has **5** item(s); client list has **4**.
- **Kids' orthodontics plan** (staged add): the client sees it as a normal record; the advisor sees it flagged `create` (and the advisor's summary leaves it out until accepted).
- **Costco Citi Visa** (staged delete): advisor still sees the committed record flagged `delete`; the client no longer sees it.
- **Minivan** (staged edit): advisor sees committed values plus `pending_change` (`owner` spouse → joint; `current_balance` 21750.00 → 20900.00); the client sees the proposed values merged in and no `pending_change` key.
- **Kitchen remodel HELOC** (staged edit): advisor sees committed values plus `pending_change` (`current_balance` 38500.00 → 36200.00; `interest_rate` 8.75 → 8.5); the client sees the proposed values merged in and no `pending_change` key.

<table>
<tr><th>Advisor GET</th><th>Client GET</th></tr>
<tr>
<td valign="top">

```json
{
  "items": [
    {
      "liability_type": "medical_debt",
      "name": "Kids' orthodontics plan",
      "lender": "Smile Dental",
      "account_last4": null,
      "owner": "joint",
      "original_balance": null,
      "current_balance": "3200.00",
      "interest_rate": null,
      "rate_type": null,
      "payment_amount": "200.00",
      "payment_frequency": "monthly",
      "origination_date": null,
      "maturity_date": null,
      "is_tax_deductible": false,
      "notes": null,
      "id": "6b00ef9a7c1b5f710ea8c504",
      "client_id": "client-family-homeowners",
      "created_at": "2026-10-26T01:10:50.228000Z",
      "updated_at": "2026-10-26T01:10:50.228000Z",
      "created_by": {
        "user_id": "cli-client-family-homeowners",
        "caller_type": "client",
        "at": "2026-10-26T01:10:50.228000Z"
      },
      "updated_by": {
        "user_id": "cli-client-family-homeowners",
        "caller_type": "client",
        "at": "2026-10-26T01:10:50.228000Z"
      },
      "pending_change": {
        "action": "create",
        "changes": {},
        "created_by": {
          "user_id": "cli-client-family-homeowners",
          "caller_type": "client",
          "at": "2026-11-18T01:10:50.228000Z"
        },
        "updated_by": {
          "user_id": "cli-client-family-homeowners",
          "caller_type": "client",
          "at": "2026-11-20T23:10:50.228000Z"
        }
      }
    },
    {
      "liability_type": "credit_card",
      "name": "Costco Citi Visa",
      "lender": "Citi",
      "account_last4": "5530",
      "owner": "joint",
      "original_balance": null,
      "current_balance": "1840.10",
      "interest_rate": "19.24",
      "rate_type": "variable",
      "payment_amount": "1840.10",
      "payment_frequency": "monthly",
      "origination_date": null,
      "maturity_date": null,
      "is_tax_deductible": false,
      "notes": "Paid in full each month.",
      "id": "6b00ef9a7c1b5f710ea8c503",
      "client_id": "client-family-homeowners",
      "created_at": "2026-10-25T01:10:50.228000Z",
      "updated_at": "2026-10-25T01:10:50.228000Z",
      "created_by": {
        "user_id": "cli-client-family-homeowners",
        "caller_type": "client",
        "at": "2026-10-25T01:10:50.228000Z"
      },
      "updated_by": {
        "user_id": "cli-client-family-homeowners",
        "caller_type": "client",
        "at": "2026-10-25T01:10:50.228000Z"
      },
      "pending_change": {
        "action": "delete",
        "changes": {},
        "created_by": {
          "user_id": "cli-client-family-homeowners",
          "caller_type": "client",
          "at": "2026-11-18T01:10:50.228000Z"
        },
        "updated_by": {
          "user_id": "cli-client-family-homeowners",
          "caller_type": "client",
          "at": "2026-11-20T23:10:50.228000Z"
        }
      }
    },
    {
      "liability_type": "auto_loan",
      "name": "Minivan",
      "lender": "Toyota Financial",
      "account_last4": null,
      "owner": "spouse",
      "original_balance": "38000",
      "current_balance": "21750.00",
      "interest_rate": "3.9",
      "rate_type": "fixed",
      "payment_amount": "610.00",
      "payment_frequency": "monthly",
      "origination_date": "2022-06-01",
      "maturity_date": "2028-06-01",
      "is_tax_deductible": false,
      "notes": null,
      "id": "6b00ef9a7c1b5f710ea8c502",
      "client_id": "client-family-homeowners",
      "created_at": "2026-10-24T01:10:50.228000Z",
      "updated_at": "2026-10-24T01:10:50.228000Z",
      "created_by": {
        "user_id": "adv-1",
        "caller_type": "advisor",
        "at": "2026-10-24T01:10:50.228000Z"
      },
      "updated_by": {
        "user_id": "adv-1",
        "caller_type": "advisor",
        "at": "2026-10-24T01:10:50.228000Z"
      },
      "pending_change": {
        "action": "update",
        "changes": {
          "owner": {
            "current": "spouse",
            "proposed": "joint"
          },
          "current_balance": {
            "current": "21750.00",
            "proposed": "20900.00"
          }
        },
        "created_by": {
          "user_id": "cli-client-family-homeowners",
          "caller_type": "client",
          "at": "2026-11-18T01:10:50.228000Z"
        },
        "updated_by": {
          "user_id": "cli-client-family-homeowners",
          "caller_type": "client",
          "at": "2026-11-20T23:10:50.228000Z"
        }
      }
    },
    {
      "liability_type": "heloc",
      "name": "Kitchen remodel HELOC",
      "lender": "Local Credit Union",
      "account_last4": null,
      "owner": "joint",
      "original_balance": "60000",
      "current_balance": "38500.00",
      "interest_rate": "8.75",
      "rate_type": "variable",
      "payment_amount": "520.00",
      "payment_frequency": "monthly",
      "origination_date": "2023-04-10",
      "maturity_date": null,
      "is_tax_deductible": true,
      "notes": null,
      "id": "6b00ef9a7c1b5f710ea8c501",
      "client_id": "client-family-homeowners",
      "created_at": "2026-10-23T01:10:50.228000Z",
      "updated_at": "2026-10-23T01:10:50.228000Z",
      "created_by": {
        "user_id": "adv-1",
        "caller_type": "advisor",
        "at": "2026-10-23T01:10:50.228000Z"
      },
      "updated_by": {
        "user_id": "adv-1",
        "caller_type": "advisor",
        "at": "2026-10-23T01:10:50.228000Z"
      },
      "pending_change": {
        "action": "update",
        "changes": {
          "current_balance": {
            "current": "38500.00",
            "proposed": "36200.00"
          },
          "interest_rate": {
            "current": "8.75",
            "proposed": "8.5"
          }
        },
        "created_by": {
          "user_id": "cli-client-family-homeowners",
          "caller_type": "client",
          "at": "2026-11-18T01:10:50.228000Z"
        },
        "updated_by": {
          "user_id": "cli-client-family-homeowners",
          "caller_type": "client",
          "at": "2026-11-20T23:10:50.228000Z"
        }
      }
    },
    {
      "liability_type": "mortgage",
      "name": "Primary residence mortgage",
      "lender": "Wells Fargo",
      "account_last4": "2288",
      "owner": "joint",
      "original_balance": "520000",
      "current_balance": "437600.00",
      "interest_rate": "3.25",
      "rate_type": "fixed",
      "payment_amount": "2263.00",
      "payment_frequency": "monthly",
      "origination_date": "2020-11-01",
      "maturity_date": "2050-11-01",
      "is_tax_deductible": true,
      "notes": null,
      "id": "6b00ef9a7c1b5f710ea8c500",
      "client_id": "client-family-homeowners",
      "created_at": "2026-10-22T01:10:50.228000Z",
      "updated_at": "2026-10-22T01:10:50.228000Z",
      "created_by": {
        "user_id": "adv-1",
        "caller_type": "advisor",
        "at": "2026-10-22T01:10:50.228000Z"
      },
      "updated_by": {
        "user_id": "adv-1",
        "caller_type": "advisor",
        "at": "2026-10-22T01:10:50.228000Z"
      },
      "pending_change": null
    }
  ],
  "total": 5
}
```

</td>
<td valign="top">

```json
{
  "items": [
    {
      "liability_type": "medical_debt",
      "name": "Kids' orthodontics plan",
      "lender": "Smile Dental",
      "account_last4": null,
      "owner": "joint",
      "original_balance": null,
      "current_balance": "3200.00",
      "interest_rate": null,
      "rate_type": null,
      "payment_amount": "200.00",
      "payment_frequency": "monthly",
      "origination_date": null,
      "maturity_date": null,
      "is_tax_deductible": false,
      "notes": null,
      "id": "6b00ef9a7c1b5f710ea8c504",
      "client_id": "client-family-homeowners",
      "created_at": "2026-10-26T01:10:50.228000Z",
      "updated_at": "2026-10-26T01:10:50.228000Z",
      "created_by": {
        "user_id": "cli-client-family-homeowners",
        "caller_type": "client",
        "at": "2026-10-26T01:10:50.228000Z"
      },
      "updated_by": {
        "user_id": "cli-client-family-homeowners",
        "caller_type": "client",
        "at": "2026-10-26T01:10:50.228000Z"
      }
    },
    {
      "liability_type": "auto_loan",
      "name": "Minivan",
      "lender": "Toyota Financial",
      "account_last4": null,
      "owner": "joint",
      "original_balance": "38000",
      "current_balance": "20900.00",
      "interest_rate": "3.9",
      "rate_type": "fixed",
      "payment_amount": "610.00",
      "payment_frequency": "monthly",
      "origination_date": "2022-06-01",
      "maturity_date": "2028-06-01",
      "is_tax_deductible": false,
      "notes": null,
      "id": "6b00ef9a7c1b5f710ea8c502",
      "client_id": "client-family-homeowners",
      "created_at": "2026-10-24T01:10:50.228000Z",
      "updated_at": "2026-10-24T01:10:50.228000Z",
      "created_by": {
        "user_id": "adv-1",
        "caller_type": "advisor",
        "at": "2026-10-24T01:10:50.228000Z"
      },
      "updated_by": {
        "user_id": "adv-1",
        "caller_type": "advisor",
        "at": "2026-10-24T01:10:50.228000Z"
      }
    },
    {
      "liability_type": "heloc",
      "name": "Kitchen remodel HELOC",
      "lender": "Local Credit Union",
      "account_last4": null,
      "owner": "joint",
      "original_balance": "60000",
      "current_balance": "36200.00",
      "interest_rate": "8.5",
      "rate_type": "variable",
      "payment_amount": "520.00",
      "payment_frequency": "monthly",
      "origination_date": "2023-04-10",
      "maturity_date": null,
      "is_tax_deductible": true,
      "notes": null,
      "id": "6b00ef9a7c1b5f710ea8c501",
      "client_id": "client-family-homeowners",
      "created_at": "2026-10-23T01:10:50.228000Z",
      "updated_at": "2026-10-23T01:10:50.228000Z",
      "created_by": {
        "user_id": "adv-1",
        "caller_type": "advisor",
        "at": "2026-10-23T01:10:50.228000Z"
      },
      "updated_by": {
        "user_id": "adv-1",
        "caller_type": "advisor",
        "at": "2026-10-23T01:10:50.228000Z"
      }
    },
    {
      "liability_type": "mortgage",
      "name": "Primary residence mortgage",
      "lender": "Wells Fargo",
      "account_last4": "2288",
      "owner": "joint",
      "original_balance": "520000",
      "current_balance": "437600.00",
      "interest_rate": "3.25",
      "rate_type": "fixed",
      "payment_amount": "2263.00",
      "payment_frequency": "monthly",
      "origination_date": "2020-11-01",
      "maturity_date": "2050-11-01",
      "is_tax_deductible": true,
      "notes": null,
      "id": "6b00ef9a7c1b5f710ea8c500",
      "client_id": "client-family-homeowners",
      "created_at": "2026-10-22T01:10:50.228000Z",
      "updated_at": "2026-10-22T01:10:50.228000Z",
      "created_by": {
        "user_id": "adv-1",
        "caller_type": "advisor",
        "at": "2026-10-22T01:10:50.228000Z"
      },
      "updated_by": {
        "user_id": "adv-1",
        "caller_type": "advisor",
        "at": "2026-10-22T01:10:50.228000Z"
      }
    }
  ],
  "total": 4
}
```

</td>
</tr>
</table>

## Edit + delete — `client-pre-retiree`

Early 60s, retiring in ~3 years: low mortgage balance, a few loose ends

Liabilities: 3 · unchanged: 1 · staged edits: 1 · adds: 0 · deletes: 1

**What to notice**

- Advisor list has **3** item(s); client list has **2**.
- **IRS installment agreement** (staged delete): advisor still sees the committed record flagged `delete`; the client no longer sees it.
- **Daughter's wedding loan** (staged edit): advisor sees committed values plus `pending_change` (`current_balance` 9400.00 → 8100.00; `payment_amount` 520.00 → 750.00; `maturity_date` 2026-12-01 → 2026-06-01); the client sees the proposed values merged in and no `pending_change` key.

<table>
<tr><th>Advisor GET</th><th>Client GET</th></tr>
<tr>
<td valign="top">

```json
{
  "items": [
    {
      "liability_type": "tax_liability",
      "name": "IRS installment agreement",
      "lender": "IRS",
      "account_last4": null,
      "owner": "client",
      "original_balance": "12000",
      "current_balance": "6000.00",
      "interest_rate": "8.0",
      "rate_type": "variable",
      "payment_amount": "500.00",
      "payment_frequency": "monthly",
      "origination_date": null,
      "maturity_date": null,
      "is_tax_deductible": false,
      "notes": "From 2023 capital gains underpayment.",
      "id": "6b00ef9a7c1b5f710ea8c507",
      "client_id": "client-pre-retiree",
      "created_at": "2026-10-24T01:10:50.228000Z",
      "updated_at": "2026-10-24T01:10:50.228000Z",
      "created_by": {
        "user_id": "cli-client-pre-retiree",
        "caller_type": "client",
        "at": "2026-10-24T01:10:50.228000Z"
      },
      "updated_by": {
        "user_id": "cli-client-pre-retiree",
        "caller_type": "client",
        "at": "2026-10-24T01:10:50.228000Z"
      },
      "pending_change": {
        "action": "delete",
        "changes": {},
        "created_by": {
          "user_id": "cli-client-pre-retiree",
          "caller_type": "client",
          "at": "2026-11-18T01:10:50.228000Z"
        },
        "updated_by": {
          "user_id": "cli-client-pre-retiree",
          "caller_type": "client",
          "at": "2026-11-20T23:10:50.228000Z"
        }
      }
    },
    {
      "liability_type": "personal_loan",
      "name": "Daughter's wedding loan",
      "lender": "SoFi",
      "account_last4": null,
      "owner": "client",
      "original_balance": "25000",
      "current_balance": "9400.00",
      "interest_rate": "9.5",
      "rate_type": "fixed",
      "payment_amount": "520.00",
      "payment_frequency": "monthly",
      "origination_date": "2022-08-01",
      "maturity_date": "2026-12-01",
      "is_tax_deductible": false,
      "notes": null,
      "id": "6b00ef9a7c1b5f710ea8c506",
      "client_id": "client-pre-retiree",
      "created_at": "2026-10-23T01:10:50.228000Z",
      "updated_at": "2026-10-23T01:10:50.228000Z",
      "created_by": {
        "user_id": "adv-1",
        "caller_type": "advisor",
        "at": "2026-10-23T01:10:50.228000Z"
      },
      "updated_by": {
        "user_id": "adv-1",
        "caller_type": "advisor",
        "at": "2026-10-23T01:10:50.228000Z"
      },
      "pending_change": {
        "action": "update",
        "changes": {
          "current_balance": {
            "current": "9400.00",
            "proposed": "8100.00"
          },
          "payment_amount": {
            "current": "520.00",
            "proposed": "750.00"
          },
          "maturity_date": {
            "current": "2026-12-01",
            "proposed": "2026-06-01"
          }
        },
        "created_by": {
          "user_id": "cli-client-pre-retiree",
          "caller_type": "client",
          "at": "2026-11-18T01:10:50.228000Z"
        },
        "updated_by": {
          "user_id": "cli-client-pre-retiree",
          "caller_type": "client",
          "at": "2026-11-20T23:10:50.228000Z"
        }
      }
    },
    {
      "liability_type": "mortgage",
      "name": "Primary residence mortgage",
      "lender": "Bank of America",
      "account_last4": null,
      "owner": "joint",
      "original_balance": "300000",
      "current_balance": "58200.00",
      "interest_rate": "2.875",
      "rate_type": "fixed",
      "payment_amount": "1450.00",
      "payment_frequency": "monthly",
      "origination_date": "2012-05-01",
      "maturity_date": "2027-05-01",
      "is_tax_deductible": true,
      "notes": null,
      "id": "6b00ef9a7c1b5f710ea8c505",
      "client_id": "client-pre-retiree",
      "created_at": "2026-10-22T01:10:50.228000Z",
      "updated_at": "2026-10-22T01:10:50.228000Z",
      "created_by": {
        "user_id": "adv-1",
        "caller_type": "advisor",
        "at": "2026-10-22T01:10:50.228000Z"
      },
      "updated_by": {
        "user_id": "adv-1",
        "caller_type": "advisor",
        "at": "2026-10-22T01:10:50.228000Z"
      },
      "pending_change": null
    }
  ],
  "total": 3
}
```

</td>
<td valign="top">

```json
{
  "items": [
    {
      "liability_type": "personal_loan",
      "name": "Daughter's wedding loan",
      "lender": "SoFi",
      "account_last4": null,
      "owner": "client",
      "original_balance": "25000",
      "current_balance": "8100.00",
      "interest_rate": "9.5",
      "rate_type": "fixed",
      "payment_amount": "750.00",
      "payment_frequency": "monthly",
      "origination_date": "2022-08-01",
      "maturity_date": "2026-06-01",
      "is_tax_deductible": false,
      "notes": null,
      "id": "6b00ef9a7c1b5f710ea8c506",
      "client_id": "client-pre-retiree",
      "created_at": "2026-10-23T01:10:50.228000Z",
      "updated_at": "2026-10-23T01:10:50.228000Z",
      "created_by": {
        "user_id": "adv-1",
        "caller_type": "advisor",
        "at": "2026-10-23T01:10:50.228000Z"
      },
      "updated_by": {
        "user_id": "adv-1",
        "caller_type": "advisor",
        "at": "2026-10-23T01:10:50.228000Z"
      }
    },
    {
      "liability_type": "mortgage",
      "name": "Primary residence mortgage",
      "lender": "Bank of America",
      "account_last4": null,
      "owner": "joint",
      "original_balance": "300000",
      "current_balance": "58200.00",
      "interest_rate": "2.875",
      "rate_type": "fixed",
      "payment_amount": "1450.00",
      "payment_frequency": "monthly",
      "origination_date": "2012-05-01",
      "maturity_date": "2027-05-01",
      "is_tax_deductible": true,
      "notes": null,
      "id": "6b00ef9a7c1b5f710ea8c505",
      "client_id": "client-pre-retiree",
      "created_at": "2026-10-22T01:10:50.228000Z",
      "updated_at": "2026-10-22T01:10:50.228000Z",
      "created_by": {
        "user_id": "adv-1",
        "caller_type": "advisor",
        "at": "2026-10-22T01:10:50.228000Z"
      },
      "updated_by": {
        "user_id": "adv-1",
        "caller_type": "advisor",
        "at": "2026-10-22T01:10:50.228000Z"
      }
    }
  ],
  "total": 2
}
```

</td>
</tr>
</table>

## Edit + add, cleared field — `client-small-business-owner`

Self-employed: business and personal debt mixed, quarterly payments

Liabilities: 5 · unchanged: 3 · staged edits: 1 · adds: 1 · deletes: 0

**What to notice**

- Advisor list has **5** item(s); client list has **5**.
- **Equipment lease** (staged add): the client sees it as a normal record; the advisor sees it flagged `create` (and the advisor's summary leaves it out until accepted).
- **SBA 7(a) loan** (staged edit): advisor sees committed values plus `pending_change` (`interest_rate` 10.5 → 9.75; `notes` Personally guaranteed. → null); the client sees the proposed values merged in and no `pending_change` key.

<table>
<tr><th>Advisor GET</th><th>Client GET</th></tr>
<tr>
<td valign="top">

```json
{
  "items": [
    {
      "liability_type": "business_loan",
      "name": "Equipment lease",
      "lender": "Dell Financial",
      "account_last4": null,
      "owner": "client",
      "original_balance": "18000",
      "current_balance": "18000.00",
      "interest_rate": "8.25",
      "rate_type": "fixed",
      "payment_amount": "560.00",
      "payment_frequency": "monthly",
      "origination_date": "2026-10-01",
      "maturity_date": "2029-10-01",
      "is_tax_deductible": true,
      "notes": null,
      "id": "6b00ef9a7c1b5f710ea8c50c",
      "client_id": "client-small-business-owner",
      "created_at": "2026-10-26T01:10:50.228000Z",
      "updated_at": "2026-10-26T01:10:50.228000Z",
      "created_by": {
        "user_id": "cli-client-small-business-owner",
        "caller_type": "client",
        "at": "2026-10-26T01:10:50.228000Z"
      },
      "updated_by": {
        "user_id": "cli-client-small-business-owner",
        "caller_type": "client",
        "at": "2026-10-26T01:10:50.228000Z"
      },
      "pending_change": {
        "action": "create",
        "changes": {},
        "created_by": {
          "user_id": "cli-client-small-business-owner",
          "caller_type": "client",
          "at": "2026-11-18T01:10:50.228000Z"
        },
        "updated_by": {
          "user_id": "cli-client-small-business-owner",
          "caller_type": "client",
          "at": "2026-11-20T23:10:50.228000Z"
        }
      }
    },
    {
      "liability_type": "mortgage",
      "name": "Primary residence mortgage",
      "lender": "Rocket Mortgage",
      "account_last4": null,
      "owner": "client",
      "original_balance": "410000",
      "current_balance": "388000.00",
      "interest_rate": "6.75",
      "rate_type": "fixed",
      "payment_amount": "2660.00",
      "payment_frequency": "monthly",
      "origination_date": "2024-01-15",
      "maturity_date": "2054-01-15",
      "is_tax_deductible": true,
      "notes": null,
      "id": "6b00ef9a7c1b5f710ea8c50b",
      "client_id": "client-small-business-owner",
      "created_at": "2026-10-25T01:10:50.228000Z",
      "updated_at": "2026-10-25T01:10:50.228000Z",
      "created_by": {
        "user_id": "adv-1",
        "caller_type": "advisor",
        "at": "2026-10-25T01:10:50.228000Z"
      },
      "updated_by": {
        "user_id": "adv-1",
        "caller_type": "advisor",
        "at": "2026-10-25T01:10:50.228000Z"
      },
      "pending_change": null
    },
    {
      "liability_type": "tax_liability",
      "name": "Estimated federal taxes",
      "lender": "IRS",
      "account_last4": null,
      "owner": "client",
      "original_balance": null,
      "current_balance": "9800.00",
      "interest_rate": null,
      "rate_type": null,
      "payment_amount": "9800.00",
      "payment_frequency": "quarterly",
      "origination_date": null,
      "maturity_date": null,
      "is_tax_deductible": false,
      "notes": null,
      "id": "6b00ef9a7c1b5f710ea8c50a",
      "client_id": "client-small-business-owner",
      "created_at": "2026-10-24T01:10:50.228000Z",
      "updated_at": "2026-10-24T01:10:50.228000Z",
      "created_by": {
        "user_id": "cli-client-small-business-owner",
        "caller_type": "client",
        "at": "2026-10-24T01:10:50.228000Z"
      },
      "updated_by": {
        "user_id": "cli-client-small-business-owner",
        "caller_type": "client",
        "at": "2026-10-24T01:10:50.228000Z"
      },
      "pending_change": null
    },
    {
      "liability_type": "credit_card",
      "name": "Amex Business Platinum",
      "lender": "American Express",
      "account_last4": "1007",
      "owner": "client",
      "original_balance": null,
      "current_balance": "12750.00",
      "interest_rate": "21.49",
      "rate_type": "variable",
      "payment_amount": "1500.00",
      "payment_frequency": "monthly",
      "origination_date": null,
      "maturity_date": null,
      "is_tax_deductible": true,
      "notes": null,
      "id": "6b00ef9a7c1b5f710ea8c509",
      "client_id": "client-small-business-owner",
      "created_at": "2026-10-23T01:10:50.228000Z",
      "updated_at": "2026-10-23T01:10:50.228000Z",
      "created_by": {
        "user_id": "adv-1",
        "caller_type": "advisor",
        "at": "2026-10-23T01:10:50.228000Z"
      },
      "updated_by": {
        "user_id": "adv-1",
        "caller_type": "advisor",
        "at": "2026-10-23T01:10:50.228000Z"
      },
      "pending_change": null
    },
    {
      "liability_type": "business_loan",
      "name": "SBA 7(a) loan",
      "lender": "Live Oak Bank",
      "account_last4": null,
      "owner": "client",
      "original_balance": "250000",
      "current_balance": "198000.00",
      "interest_rate": "10.5",
      "rate_type": "variable",
      "payment_amount": "3300.00",
      "payment_frequency": "monthly",
      "origination_date": "2021-02-01",
      "maturity_date": "2031-02-01",
      "is_tax_deductible": true,
      "notes": "Personally guaranteed.",
      "id": "6b00ef9a7c1b5f710ea8c508",
      "client_id": "client-small-business-owner",
      "created_at": "2026-10-22T01:10:50.228000Z",
      "updated_at": "2026-10-22T01:10:50.228000Z",
      "created_by": {
        "user_id": "adv-1",
        "caller_type": "advisor",
        "at": "2026-10-22T01:10:50.228000Z"
      },
      "updated_by": {
        "user_id": "adv-1",
        "caller_type": "advisor",
        "at": "2026-10-22T01:10:50.228000Z"
      },
      "pending_change": {
        "action": "update",
        "changes": {
          "interest_rate": {
            "current": "10.5",
            "proposed": "9.75"
          },
          "notes": {
            "current": "Personally guaranteed.",
            "proposed": null
          }
        },
        "created_by": {
          "user_id": "cli-client-small-business-owner",
          "caller_type": "client",
          "at": "2026-11-18T01:10:50.228000Z"
        },
        "updated_by": {
          "user_id": "cli-client-small-business-owner",
          "caller_type": "client",
          "at": "2026-11-20T23:10:50.228000Z"
        }
      }
    }
  ],
  "total": 5
}
```

</td>
<td valign="top">

```json
{
  "items": [
    {
      "liability_type": "business_loan",
      "name": "Equipment lease",
      "lender": "Dell Financial",
      "account_last4": null,
      "owner": "client",
      "original_balance": "18000",
      "current_balance": "18000.00",
      "interest_rate": "8.25",
      "rate_type": "fixed",
      "payment_amount": "560.00",
      "payment_frequency": "monthly",
      "origination_date": "2026-10-01",
      "maturity_date": "2029-10-01",
      "is_tax_deductible": true,
      "notes": null,
      "id": "6b00ef9a7c1b5f710ea8c50c",
      "client_id": "client-small-business-owner",
      "created_at": "2026-10-26T01:10:50.228000Z",
      "updated_at": "2026-10-26T01:10:50.228000Z",
      "created_by": {
        "user_id": "cli-client-small-business-owner",
        "caller_type": "client",
        "at": "2026-10-26T01:10:50.228000Z"
      },
      "updated_by": {
        "user_id": "cli-client-small-business-owner",
        "caller_type": "client",
        "at": "2026-10-26T01:10:50.228000Z"
      }
    },
    {
      "liability_type": "mortgage",
      "name": "Primary residence mortgage",
      "lender": "Rocket Mortgage",
      "account_last4": null,
      "owner": "client",
      "original_balance": "410000",
      "current_balance": "388000.00",
      "interest_rate": "6.75",
      "rate_type": "fixed",
      "payment_amount": "2660.00",
      "payment_frequency": "monthly",
      "origination_date": "2024-01-15",
      "maturity_date": "2054-01-15",
      "is_tax_deductible": true,
      "notes": null,
      "id": "6b00ef9a7c1b5f710ea8c50b",
      "client_id": "client-small-business-owner",
      "created_at": "2026-10-25T01:10:50.228000Z",
      "updated_at": "2026-10-25T01:10:50.228000Z",
      "created_by": {
        "user_id": "adv-1",
        "caller_type": "advisor",
        "at": "2026-10-25T01:10:50.228000Z"
      },
      "updated_by": {
        "user_id": "adv-1",
        "caller_type": "advisor",
        "at": "2026-10-25T01:10:50.228000Z"
      }
    },
    {
      "liability_type": "tax_liability",
      "name": "Estimated federal taxes",
      "lender": "IRS",
      "account_last4": null,
      "owner": "client",
      "original_balance": null,
      "current_balance": "9800.00",
      "interest_rate": null,
      "rate_type": null,
      "payment_amount": "9800.00",
      "payment_frequency": "quarterly",
      "origination_date": null,
      "maturity_date": null,
      "is_tax_deductible": false,
      "notes": null,
      "id": "6b00ef9a7c1b5f710ea8c50a",
      "client_id": "client-small-business-owner",
      "created_at": "2026-10-24T01:10:50.228000Z",
      "updated_at": "2026-10-24T01:10:50.228000Z",
      "created_by": {
        "user_id": "cli-client-small-business-owner",
        "caller_type": "client",
        "at": "2026-10-24T01:10:50.228000Z"
      },
      "updated_by": {
        "user_id": "cli-client-small-business-owner",
        "caller_type": "client",
        "at": "2026-10-24T01:10:50.228000Z"
      }
    },
    {
      "liability_type": "credit_card",
      "name": "Amex Business Platinum",
      "lender": "American Express",
      "account_last4": "1007",
      "owner": "client",
      "original_balance": null,
      "current_balance": "12750.00",
      "interest_rate": "21.49",
      "rate_type": "variable",
      "payment_amount": "1500.00",
      "payment_frequency": "monthly",
      "origination_date": null,
      "maturity_date": null,
      "is_tax_deductible": true,
      "notes": null,
      "id": "6b00ef9a7c1b5f710ea8c509",
      "client_id": "client-small-business-owner",
      "created_at": "2026-10-23T01:10:50.228000Z",
      "updated_at": "2026-10-23T01:10:50.228000Z",
      "created_by": {
        "user_id": "adv-1",
        "caller_type": "advisor",
        "at": "2026-10-23T01:10:50.228000Z"
      },
      "updated_by": {
        "user_id": "adv-1",
        "caller_type": "advisor",
        "at": "2026-10-23T01:10:50.228000Z"
      }
    },
    {
      "liability_type": "business_loan",
      "name": "SBA 7(a) loan",
      "lender": "Live Oak Bank",
      "account_last4": null,
      "owner": "client",
      "original_balance": "250000",
      "current_balance": "198000.00",
      "interest_rate": "9.75",
      "rate_type": "variable",
      "payment_amount": "3300.00",
      "payment_frequency": "monthly",
      "origination_date": "2021-02-01",
      "maturity_date": "2031-02-01",
      "is_tax_deductible": true,
      "notes": null,
      "id": "6b00ef9a7c1b5f710ea8c508",
      "client_id": "client-small-business-owner",
      "created_at": "2026-10-22T01:10:50.228000Z",
      "updated_at": "2026-10-22T01:10:50.228000Z",
      "created_by": {
        "user_id": "adv-1",
        "caller_type": "advisor",
        "at": "2026-10-22T01:10:50.228000Z"
      },
      "updated_by": {
        "user_id": "adv-1",
        "caller_type": "advisor",
        "at": "2026-10-22T01:10:50.228000Z"
      }
    }
  ],
  "total": 5
}
```

</td>
</tr>
</table>

## Edit + delete, high balances — `client-medical-professional`

Recent physician: very high student debt, medical practice loan, rental property

Liabilities: 4 · unchanged: 2 · staged edits: 1 · adds: 0 · deletes: 1

**What to notice**

- Advisor list has **4** item(s); client list has **3**.
- **Outstanding hospital bill** (staged delete): advisor still sees the committed record flagged `delete`; the client no longer sees it.
- **Rental property mortgage** (staged edit): advisor sees committed values plus `pending_change` (`lender` Chase → Wells Fargo; `owner` client → joint; `interest_rate` 7.1 → 6.4); the client sees the proposed values merged in and no `pending_change` key.

<table>
<tr><th>Advisor GET</th><th>Client GET</th></tr>
<tr>
<td valign="top">

```json
{
  "items": [
    {
      "liability_type": "medical_debt",
      "name": "Outstanding hospital bill",
      "lender": "County Hospital",
      "account_last4": null,
      "owner": "client",
      "original_balance": null,
      "current_balance": "2300.00",
      "interest_rate": null,
      "rate_type": null,
      "payment_amount": "200.00",
      "payment_frequency": "monthly",
      "origination_date": null,
      "maturity_date": null,
      "is_tax_deductible": false,
      "notes": null,
      "id": "6b00ef9a7c1b5f710ea8c510",
      "client_id": "client-medical-professional",
      "created_at": "2026-10-25T01:10:50.228000Z",
      "updated_at": "2026-10-25T01:10:50.228000Z",
      "created_by": {
        "user_id": "cli-client-medical-professional",
        "caller_type": "client",
        "at": "2026-10-25T01:10:50.228000Z"
      },
      "updated_by": {
        "user_id": "cli-client-medical-professional",
        "caller_type": "client",
        "at": "2026-10-25T01:10:50.228000Z"
      },
      "pending_change": {
        "action": "delete",
        "changes": {},
        "created_by": {
          "user_id": "cli-client-medical-professional",
          "caller_type": "client",
          "at": "2026-11-18T01:10:50.228000Z"
        },
        "updated_by": {
          "user_id": "cli-client-medical-professional",
          "caller_type": "client",
          "at": "2026-11-20T23:10:50.228000Z"
        }
      }
    },
    {
      "liability_type": "mortgage",
      "name": "Rental property mortgage",
      "lender": "Chase",
      "account_last4": "7765",
      "owner": "client",
      "original_balance": "275000",
      "current_balance": "268900.00",
      "interest_rate": "7.1",
      "rate_type": "fixed",
      "payment_amount": "1850.00",
      "payment_frequency": "monthly",
      "origination_date": "2024-06-01",
      "maturity_date": "2054-06-01",
      "is_tax_deductible": true,
      "notes": null,
      "id": "6b00ef9a7c1b5f710ea8c50f",
      "client_id": "client-medical-professional",
      "created_at": "2026-10-24T01:10:50.228000Z",
      "updated_at": "2026-10-24T01:10:50.228000Z",
      "created_by": {
        "user_id": "adv-1",
        "caller_type": "advisor",
        "at": "2026-10-24T01:10:50.228000Z"
      },
      "updated_by": {
        "user_id": "adv-1",
        "caller_type": "advisor",
        "at": "2026-10-24T01:10:50.228000Z"
      },
      "pending_change": {
        "action": "update",
        "changes": {
          "lender": {
            "current": "Chase",
            "proposed": "Wells Fargo"
          },
          "owner": {
            "current": "client",
            "proposed": "joint"
          },
          "interest_rate": {
            "current": "7.1",
            "proposed": "6.4"
          }
        },
        "created_by": {
          "user_id": "cli-client-medical-professional",
          "caller_type": "client",
          "at": "2026-11-18T01:10:50.228000Z"
        },
        "updated_by": {
          "user_id": "cli-client-medical-professional",
          "caller_type": "client",
          "at": "2026-11-20T23:10:50.228000Z"
        }
      }
    },
    {
      "liability_type": "student_loan",
      "name": "Federal Grad PLUS",
      "lender": "Dept. of Education",
      "account_last4": null,
      "owner": "client",
      "original_balance": "120000",
      "current_balance": "131900.00",
      "interest_rate": "6.28",
      "rate_type": "fixed",
      "payment_amount": "0.00",
      "payment_frequency": "monthly",
      "origination_date": null,
      "maturity_date": null,
      "is_tax_deductible": true,
      "notes": "Forbearance during residency.",
      "id": "6b00ef9a7c1b5f710ea8c50e",
      "client_id": "client-medical-professional",
      "created_at": "2026-10-23T01:10:50.228000Z",
      "updated_at": "2026-10-23T01:10:50.228000Z",
      "created_by": {
        "user_id": "adv-1",
        "caller_type": "advisor",
        "at": "2026-10-23T01:10:50.228000Z"
      },
      "updated_by": {
        "user_id": "adv-1",
        "caller_type": "advisor",
        "at": "2026-10-23T01:10:50.228000Z"
      },
      "pending_change": null
    },
    {
      "liability_type": "student_loan",
      "name": "Medical school loans (private)",
      "lender": "Sallie Mae",
      "account_last4": null,
      "owner": "client",
      "original_balance": "280000",
      "current_balance": "305400.00",
      "interest_rate": "7.2",
      "rate_type": "variable",
      "payment_amount": "900.00",
      "payment_frequency": "monthly",
      "origination_date": "2016-08-01",
      "maturity_date": null,
      "is_tax_deductible": true,
      "notes": "In income-driven repayment; balance still growing.",
      "id": "6b00ef9a7c1b5f710ea8c50d",
      "client_id": "client-medical-professional",
      "created_at": "2026-10-22T01:10:50.228000Z",
      "updated_at": "2026-10-22T01:10:50.228000Z",
      "created_by": {
        "user_id": "adv-1",
        "caller_type": "advisor",
        "at": "2026-10-22T01:10:50.228000Z"
      },
      "updated_by": {
        "user_id": "adv-1",
        "caller_type": "advisor",
        "at": "2026-10-22T01:10:50.228000Z"
      },
      "pending_change": null
    }
  ],
  "total": 4
}
```

</td>
<td valign="top">

```json
{
  "items": [
    {
      "liability_type": "mortgage",
      "name": "Rental property mortgage",
      "lender": "Wells Fargo",
      "account_last4": "7765",
      "owner": "joint",
      "original_balance": "275000",
      "current_balance": "268900.00",
      "interest_rate": "6.4",
      "rate_type": "fixed",
      "payment_amount": "1850.00",
      "payment_frequency": "monthly",
      "origination_date": "2024-06-01",
      "maturity_date": "2054-06-01",
      "is_tax_deductible": true,
      "notes": null,
      "id": "6b00ef9a7c1b5f710ea8c50f",
      "client_id": "client-medical-professional",
      "created_at": "2026-10-24T01:10:50.228000Z",
      "updated_at": "2026-10-24T01:10:50.228000Z",
      "created_by": {
        "user_id": "adv-1",
        "caller_type": "advisor",
        "at": "2026-10-24T01:10:50.228000Z"
      },
      "updated_by": {
        "user_id": "adv-1",
        "caller_type": "advisor",
        "at": "2026-10-24T01:10:50.228000Z"
      }
    },
    {
      "liability_type": "student_loan",
      "name": "Federal Grad PLUS",
      "lender": "Dept. of Education",
      "account_last4": null,
      "owner": "client",
      "original_balance": "120000",
      "current_balance": "131900.00",
      "interest_rate": "6.28",
      "rate_type": "fixed",
      "payment_amount": "0.00",
      "payment_frequency": "monthly",
      "origination_date": null,
      "maturity_date": null,
      "is_tax_deductible": true,
      "notes": "Forbearance during residency.",
      "id": "6b00ef9a7c1b5f710ea8c50e",
      "client_id": "client-medical-professional",
      "created_at": "2026-10-23T01:10:50.228000Z",
      "updated_at": "2026-10-23T01:10:50.228000Z",
      "created_by": {
        "user_id": "adv-1",
        "caller_type": "advisor",
        "at": "2026-10-23T01:10:50.228000Z"
      },
      "updated_by": {
        "user_id": "adv-1",
        "caller_type": "advisor",
        "at": "2026-10-23T01:10:50.228000Z"
      }
    },
    {
      "liability_type": "student_loan",
      "name": "Medical school loans (private)",
      "lender": "Sallie Mae",
      "account_last4": null,
      "owner": "client",
      "original_balance": "280000",
      "current_balance": "305400.00",
      "interest_rate": "7.2",
      "rate_type": "variable",
      "payment_amount": "900.00",
      "payment_frequency": "monthly",
      "origination_date": "2016-08-01",
      "maturity_date": null,
      "is_tax_deductible": true,
      "notes": "In income-driven repayment; balance still growing.",
      "id": "6b00ef9a7c1b5f710ea8c50d",
      "client_id": "client-medical-professional",
      "created_at": "2026-10-22T01:10:50.228000Z",
      "updated_at": "2026-10-22T01:10:50.228000Z",
      "created_by": {
        "user_id": "adv-1",
        "caller_type": "advisor",
        "at": "2026-10-22T01:10:50.228000Z"
      },
      "updated_by": {
        "user_id": "adv-1",
        "caller_type": "advisor",
        "at": "2026-10-22T01:10:50.228000Z"
      }
    }
  ],
  "total": 3
}
```

</td>
</tr>
</table>

## No staged changes — `client-no-pending`

Nothing staged. Advisor and client views are identical and no pending_change appears.

Liabilities: 3 · unchanged: 3 · staged edits: 0 · adds: 0 · deletes: 0

**What to notice**

- Advisor list has **3** item(s); client list has **3**.
- Nothing is staged, so the two views are identical and no `pending_change` appears.

<table>
<tr><th>Advisor GET</th><th>Client GET</th></tr>
<tr>
<td valign="top">

```json
{
  "items": [
    {
      "liability_type": "credit_card",
      "name": "Everyday Mastercard",
      "lender": "Citi",
      "account_last4": null,
      "owner": "client",
      "original_balance": null,
      "current_balance": "860.00",
      "interest_rate": "20.5",
      "rate_type": "variable",
      "payment_amount": "860.00",
      "payment_frequency": "monthly",
      "origination_date": null,
      "maturity_date": null,
      "is_tax_deductible": false,
      "notes": null,
      "id": "6b00ef9a7c1b5f710ea8c513",
      "client_id": "client-no-pending",
      "created_at": "2026-10-24T01:10:50.228000Z",
      "updated_at": "2026-10-24T01:10:50.228000Z",
      "created_by": {
        "user_id": "cli-client-no-pending",
        "caller_type": "client",
        "at": "2026-10-24T01:10:50.228000Z"
      },
      "updated_by": {
        "user_id": "cli-client-no-pending",
        "caller_type": "client",
        "at": "2026-10-24T01:10:50.228000Z"
      },
      "pending_change": null
    },
    {
      "liability_type": "auto_loan",
      "name": "Subaru Outback",
      "lender": "Subaru Motors Finance",
      "account_last4": null,
      "owner": "client",
      "original_balance": "30000",
      "current_balance": "18400.00",
      "interest_rate": "3.5",
      "rate_type": "fixed",
      "payment_amount": "540.00",
      "payment_frequency": "monthly",
      "origination_date": null,
      "maturity_date": null,
      "is_tax_deductible": false,
      "notes": null,
      "id": "6b00ef9a7c1b5f710ea8c512",
      "client_id": "client-no-pending",
      "created_at": "2026-10-23T01:10:50.228000Z",
      "updated_at": "2026-10-23T01:10:50.228000Z",
      "created_by": {
        "user_id": "adv-1",
        "caller_type": "advisor",
        "at": "2026-10-23T01:10:50.228000Z"
      },
      "updated_by": {
        "user_id": "adv-1",
        "caller_type": "advisor",
        "at": "2026-10-23T01:10:50.228000Z"
      },
      "pending_change": null
    },
    {
      "liability_type": "mortgage",
      "name": "Primary residence mortgage",
      "lender": "US Bank",
      "account_last4": null,
      "owner": "client",
      "original_balance": "350000",
      "current_balance": "312000.00",
      "interest_rate": "4.1",
      "rate_type": "fixed",
      "payment_amount": "1690.00",
      "payment_frequency": "monthly",
      "origination_date": "2021-07-01",
      "maturity_date": "2051-07-01",
      "is_tax_deductible": true,
      "notes": null,
      "id": "6b00ef9a7c1b5f710ea8c511",
      "client_id": "client-no-pending",
      "created_at": "2026-10-22T01:10:50.228000Z",
      "updated_at": "2026-10-22T01:10:50.228000Z",
      "created_by": {
        "user_id": "adv-1",
        "caller_type": "advisor",
        "at": "2026-10-22T01:10:50.228000Z"
      },
      "updated_by": {
        "user_id": "adv-1",
        "caller_type": "advisor",
        "at": "2026-10-22T01:10:50.228000Z"
      },
      "pending_change": null
    }
  ],
  "total": 3
}
```

</td>
<td valign="top">

```json
{
  "items": [
    {
      "liability_type": "credit_card",
      "name": "Everyday Mastercard",
      "lender": "Citi",
      "account_last4": null,
      "owner": "client",
      "original_balance": null,
      "current_balance": "860.00",
      "interest_rate": "20.5",
      "rate_type": "variable",
      "payment_amount": "860.00",
      "payment_frequency": "monthly",
      "origination_date": null,
      "maturity_date": null,
      "is_tax_deductible": false,
      "notes": null,
      "id": "6b00ef9a7c1b5f710ea8c513",
      "client_id": "client-no-pending",
      "created_at": "2026-10-24T01:10:50.228000Z",
      "updated_at": "2026-10-24T01:10:50.228000Z",
      "created_by": {
        "user_id": "cli-client-no-pending",
        "caller_type": "client",
        "at": "2026-10-24T01:10:50.228000Z"
      },
      "updated_by": {
        "user_id": "cli-client-no-pending",
        "caller_type": "client",
        "at": "2026-10-24T01:10:50.228000Z"
      }
    },
    {
      "liability_type": "auto_loan",
      "name": "Subaru Outback",
      "lender": "Subaru Motors Finance",
      "account_last4": null,
      "owner": "client",
      "original_balance": "30000",
      "current_balance": "18400.00",
      "interest_rate": "3.5",
      "rate_type": "fixed",
      "payment_amount": "540.00",
      "payment_frequency": "monthly",
      "origination_date": null,
      "maturity_date": null,
      "is_tax_deductible": false,
      "notes": null,
      "id": "6b00ef9a7c1b5f710ea8c512",
      "client_id": "client-no-pending",
      "created_at": "2026-10-23T01:10:50.228000Z",
      "updated_at": "2026-10-23T01:10:50.228000Z",
      "created_by": {
        "user_id": "adv-1",
        "caller_type": "advisor",
        "at": "2026-10-23T01:10:50.228000Z"
      },
      "updated_by": {
        "user_id": "adv-1",
        "caller_type": "advisor",
        "at": "2026-10-23T01:10:50.228000Z"
      }
    },
    {
      "liability_type": "mortgage",
      "name": "Primary residence mortgage",
      "lender": "US Bank",
      "account_last4": null,
      "owner": "client",
      "original_balance": "350000",
      "current_balance": "312000.00",
      "interest_rate": "4.1",
      "rate_type": "fixed",
      "payment_amount": "1690.00",
      "payment_frequency": "monthly",
      "origination_date": "2021-07-01",
      "maturity_date": "2051-07-01",
      "is_tax_deductible": true,
      "notes": null,
      "id": "6b00ef9a7c1b5f710ea8c511",
      "client_id": "client-no-pending",
      "created_at": "2026-10-22T01:10:50.228000Z",
      "updated_at": "2026-10-22T01:10:50.228000Z",
      "created_by": {
        "user_id": "adv-1",
        "caller_type": "advisor",
        "at": "2026-10-22T01:10:50.228000Z"
      },
      "updated_by": {
        "user_id": "adv-1",
        "caller_type": "advisor",
        "at": "2026-10-22T01:10:50.228000Z"
      }
    }
  ],
  "total": 3
}
```

</td>
</tr>
</table>

## Staged edits only — `client-edits-only`

One untouched liability and two with staged client edits, one changing several fields.

Liabilities: 3 · unchanged: 1 · staged edits: 2 · adds: 0 · deletes: 0

**What to notice**

- Advisor list has **3** item(s); client list has **3**.
- **Travel Amex** (staged edit): advisor sees committed values plus `pending_change` (`current_balance` 4200.00 → 3500.00); the client sees the proposed values merged in and no `pending_change` key.
- **Grad school loan** (staged edit): advisor sees committed values plus `pending_change` (`current_balance` 47500.00 → 45000.00; `interest_rate` 6.8 → 6.2; `payment_amount` 560.00 → 700.00); the client sees the proposed values merged in and no `pending_change` key.

<table>
<tr><th>Advisor GET</th><th>Client GET</th></tr>
<tr>
<td valign="top">

```json
{
  "items": [
    {
      "liability_type": "credit_card",
      "name": "Travel Amex",
      "lender": "American Express",
      "account_last4": null,
      "owner": "client",
      "original_balance": null,
      "current_balance": "4200.00",
      "interest_rate": "24.0",
      "rate_type": "variable",
      "payment_amount": "300.00",
      "payment_frequency": "monthly",
      "origination_date": null,
      "maturity_date": null,
      "is_tax_deductible": false,
      "notes": null,
      "id": "6b00ef9a7c1b5f710ea8c516",
      "client_id": "client-edits-only",
      "created_at": "2026-10-24T01:10:50.228000Z",
      "updated_at": "2026-10-24T01:10:50.228000Z",
      "created_by": {
        "user_id": "cli-client-edits-only",
        "caller_type": "client",
        "at": "2026-10-24T01:10:50.228000Z"
      },
      "updated_by": {
        "user_id": "cli-client-edits-only",
        "caller_type": "client",
        "at": "2026-10-24T01:10:50.228000Z"
      },
      "pending_change": {
        "action": "update",
        "changes": {
          "current_balance": {
            "current": "4200.00",
            "proposed": "3500.00"
          }
        },
        "created_by": {
          "user_id": "cli-client-edits-only",
          "caller_type": "client",
          "at": "2026-11-18T01:10:50.228000Z"
        },
        "updated_by": {
          "user_id": "cli-client-edits-only",
          "caller_type": "client",
          "at": "2026-11-20T23:10:50.228000Z"
        }
      }
    },
    {
      "liability_type": "student_loan",
      "name": "Grad school loan",
      "lender": "Navient",
      "account_last4": null,
      "owner": "client",
      "original_balance": "60000",
      "current_balance": "47500.00",
      "interest_rate": "6.8",
      "rate_type": "fixed",
      "payment_amount": "560.00",
      "payment_frequency": "monthly",
      "origination_date": null,
      "maturity_date": null,
      "is_tax_deductible": false,
      "notes": null,
      "id": "6b00ef9a7c1b5f710ea8c515",
      "client_id": "client-edits-only",
      "created_at": "2026-10-23T01:10:50.228000Z",
      "updated_at": "2026-10-23T01:10:50.228000Z",
      "created_by": {
        "user_id": "adv-1",
        "caller_type": "advisor",
        "at": "2026-10-23T01:10:50.228000Z"
      },
      "updated_by": {
        "user_id": "adv-1",
        "caller_type": "advisor",
        "at": "2026-10-23T01:10:50.228000Z"
      },
      "pending_change": {
        "action": "update",
        "changes": {
          "current_balance": {
            "current": "47500.00",
            "proposed": "45000.00"
          },
          "interest_rate": {
            "current": "6.8",
            "proposed": "6.2"
          },
          "payment_amount": {
            "current": "560.00",
            "proposed": "700.00"
          }
        },
        "created_by": {
          "user_id": "cli-client-edits-only",
          "caller_type": "client",
          "at": "2026-11-18T01:10:50.228000Z"
        },
        "updated_by": {
          "user_id": "cli-client-edits-only",
          "caller_type": "client",
          "at": "2026-11-20T23:10:50.228000Z"
        }
      }
    },
    {
      "liability_type": "auto_loan",
      "name": "Ford F-150",
      "lender": "Ford Credit",
      "account_last4": null,
      "owner": "client",
      "original_balance": "52000",
      "current_balance": "41000.00",
      "interest_rate": "5.9",
      "rate_type": "fixed",
      "payment_amount": "780.00",
      "payment_frequency": "monthly",
      "origination_date": null,
      "maturity_date": null,
      "is_tax_deductible": false,
      "notes": null,
      "id": "6b00ef9a7c1b5f710ea8c514",
      "client_id": "client-edits-only",
      "created_at": "2026-10-22T01:10:50.228000Z",
      "updated_at": "2026-10-22T01:10:50.228000Z",
      "created_by": {
        "user_id": "adv-1",
        "caller_type": "advisor",
        "at": "2026-10-22T01:10:50.228000Z"
      },
      "updated_by": {
        "user_id": "adv-1",
        "caller_type": "advisor",
        "at": "2026-10-22T01:10:50.228000Z"
      },
      "pending_change": null
    }
  ],
  "total": 3
}
```

</td>
<td valign="top">

```json
{
  "items": [
    {
      "liability_type": "credit_card",
      "name": "Travel Amex",
      "lender": "American Express",
      "account_last4": null,
      "owner": "client",
      "original_balance": null,
      "current_balance": "3500.00",
      "interest_rate": "24.0",
      "rate_type": "variable",
      "payment_amount": "300.00",
      "payment_frequency": "monthly",
      "origination_date": null,
      "maturity_date": null,
      "is_tax_deductible": false,
      "notes": null,
      "id": "6b00ef9a7c1b5f710ea8c516",
      "client_id": "client-edits-only",
      "created_at": "2026-10-24T01:10:50.228000Z",
      "updated_at": "2026-10-24T01:10:50.228000Z",
      "created_by": {
        "user_id": "cli-client-edits-only",
        "caller_type": "client",
        "at": "2026-10-24T01:10:50.228000Z"
      },
      "updated_by": {
        "user_id": "cli-client-edits-only",
        "caller_type": "client",
        "at": "2026-10-24T01:10:50.228000Z"
      }
    },
    {
      "liability_type": "student_loan",
      "name": "Grad school loan",
      "lender": "Navient",
      "account_last4": null,
      "owner": "client",
      "original_balance": "60000",
      "current_balance": "45000.00",
      "interest_rate": "6.2",
      "rate_type": "fixed",
      "payment_amount": "700.00",
      "payment_frequency": "monthly",
      "origination_date": null,
      "maturity_date": null,
      "is_tax_deductible": false,
      "notes": null,
      "id": "6b00ef9a7c1b5f710ea8c515",
      "client_id": "client-edits-only",
      "created_at": "2026-10-23T01:10:50.228000Z",
      "updated_at": "2026-10-23T01:10:50.228000Z",
      "created_by": {
        "user_id": "adv-1",
        "caller_type": "advisor",
        "at": "2026-10-23T01:10:50.228000Z"
      },
      "updated_by": {
        "user_id": "adv-1",
        "caller_type": "advisor",
        "at": "2026-10-23T01:10:50.228000Z"
      }
    },
    {
      "liability_type": "auto_loan",
      "name": "Ford F-150",
      "lender": "Ford Credit",
      "account_last4": null,
      "owner": "client",
      "original_balance": "52000",
      "current_balance": "41000.00",
      "interest_rate": "5.9",
      "rate_type": "fixed",
      "payment_amount": "780.00",
      "payment_frequency": "monthly",
      "origination_date": null,
      "maturity_date": null,
      "is_tax_deductible": false,
      "notes": null,
      "id": "6b00ef9a7c1b5f710ea8c514",
      "client_id": "client-edits-only",
      "created_at": "2026-10-22T01:10:50.228000Z",
      "updated_at": "2026-10-22T01:10:50.228000Z",
      "created_by": {
        "user_id": "adv-1",
        "caller_type": "advisor",
        "at": "2026-10-22T01:10:50.228000Z"
      },
      "updated_by": {
        "user_id": "adv-1",
        "caller_type": "advisor",
        "at": "2026-10-22T01:10:50.228000Z"
      }
    }
  ],
  "total": 3
}
```

</td>
</tr>
</table>

## Staged adds only, new client — `client-adds-only`

A new client with nothing committed. Both liabilities are pending adds, so advisor totals are 0 while client totals are not.

Liabilities: 2 · unchanged: 0 · staged edits: 0 · adds: 2 · deletes: 0

**What to notice**

- Advisor list has **2** item(s); client list has **2**.
- **Debt consolidation loan** (staged add): the client sees it as a normal record; the advisor sees it flagged `create` (and the advisor's summary leaves it out until accepted).
- **Discover It** (staged add): the client sees it as a normal record; the advisor sees it flagged `create` (and the advisor's summary leaves it out until accepted).

<table>
<tr><th>Advisor GET</th><th>Client GET</th></tr>
<tr>
<td valign="top">

```json
{
  "items": [
    {
      "liability_type": "personal_loan",
      "name": "Debt consolidation loan",
      "lender": "LendingClub",
      "account_last4": null,
      "owner": "client",
      "original_balance": "9000",
      "current_balance": "8600.00",
      "interest_rate": "11.2",
      "rate_type": "fixed",
      "payment_amount": "295.00",
      "payment_frequency": "monthly",
      "origination_date": null,
      "maturity_date": null,
      "is_tax_deductible": false,
      "notes": null,
      "id": "6b00ef9a7c1b5f710ea8c518",
      "client_id": "client-adds-only",
      "created_at": "2026-10-23T01:10:50.228000Z",
      "updated_at": "2026-10-23T01:10:50.228000Z",
      "created_by": {
        "user_id": "cli-client-adds-only",
        "caller_type": "client",
        "at": "2026-10-23T01:10:50.228000Z"
      },
      "updated_by": {
        "user_id": "cli-client-adds-only",
        "caller_type": "client",
        "at": "2026-10-23T01:10:50.228000Z"
      },
      "pending_change": {
        "action": "create",
        "changes": {},
        "created_by": {
          "user_id": "cli-client-adds-only",
          "caller_type": "client",
          "at": "2026-11-18T01:10:50.228000Z"
        },
        "updated_by": {
          "user_id": "cli-client-adds-only",
          "caller_type": "client",
          "at": "2026-11-20T23:10:50.228000Z"
        }
      }
    },
    {
      "liability_type": "credit_card",
      "name": "Discover It",
      "lender": "Discover",
      "account_last4": null,
      "owner": "client",
      "original_balance": null,
      "current_balance": "2750.00",
      "interest_rate": "17.9",
      "rate_type": "variable",
      "payment_amount": "120.00",
      "payment_frequency": "monthly",
      "origination_date": null,
      "maturity_date": null,
      "is_tax_deductible": false,
      "notes": null,
      "id": "6b00ef9a7c1b5f710ea8c517",
      "client_id": "client-adds-only",
      "created_at": "2026-10-22T01:10:50.228000Z",
      "updated_at": "2026-10-22T01:10:50.228000Z",
      "created_by": {
        "user_id": "cli-client-adds-only",
        "caller_type": "client",
        "at": "2026-10-22T01:10:50.228000Z"
      },
      "updated_by": {
        "user_id": "cli-client-adds-only",
        "caller_type": "client",
        "at": "2026-10-22T01:10:50.228000Z"
      },
      "pending_change": {
        "action": "create",
        "changes": {},
        "created_by": {
          "user_id": "cli-client-adds-only",
          "caller_type": "client",
          "at": "2026-11-18T01:10:50.228000Z"
        },
        "updated_by": {
          "user_id": "cli-client-adds-only",
          "caller_type": "client",
          "at": "2026-11-20T23:10:50.228000Z"
        }
      }
    }
  ],
  "total": 2
}
```

</td>
<td valign="top">

```json
{
  "items": [
    {
      "liability_type": "personal_loan",
      "name": "Debt consolidation loan",
      "lender": "LendingClub",
      "account_last4": null,
      "owner": "client",
      "original_balance": "9000",
      "current_balance": "8600.00",
      "interest_rate": "11.2",
      "rate_type": "fixed",
      "payment_amount": "295.00",
      "payment_frequency": "monthly",
      "origination_date": null,
      "maturity_date": null,
      "is_tax_deductible": false,
      "notes": null,
      "id": "6b00ef9a7c1b5f710ea8c518",
      "client_id": "client-adds-only",
      "created_at": "2026-10-23T01:10:50.228000Z",
      "updated_at": "2026-10-23T01:10:50.228000Z",
      "created_by": {
        "user_id": "cli-client-adds-only",
        "caller_type": "client",
        "at": "2026-10-23T01:10:50.228000Z"
      },
      "updated_by": {
        "user_id": "cli-client-adds-only",
        "caller_type": "client",
        "at": "2026-10-23T01:10:50.228000Z"
      }
    },
    {
      "liability_type": "credit_card",
      "name": "Discover It",
      "lender": "Discover",
      "account_last4": null,
      "owner": "client",
      "original_balance": null,
      "current_balance": "2750.00",
      "interest_rate": "17.9",
      "rate_type": "variable",
      "payment_amount": "120.00",
      "payment_frequency": "monthly",
      "origination_date": null,
      "maturity_date": null,
      "is_tax_deductible": false,
      "notes": null,
      "id": "6b00ef9a7c1b5f710ea8c517",
      "client_id": "client-adds-only",
      "created_at": "2026-10-22T01:10:50.228000Z",
      "updated_at": "2026-10-22T01:10:50.228000Z",
      "created_by": {
        "user_id": "cli-client-adds-only",
        "caller_type": "client",
        "at": "2026-10-22T01:10:50.228000Z"
      },
      "updated_by": {
        "user_id": "cli-client-adds-only",
        "caller_type": "client",
        "at": "2026-10-22T01:10:50.228000Z"
      }
    }
  ],
  "total": 2
}
```

</td>
</tr>
</table>

## Single staged add — `client-single-add`

A new client whose only liability is one pending add. The advisor sees it flagged create with totals of 0, while the client sees it as a normal record.

Liabilities: 1 · unchanged: 0 · staged edits: 0 · adds: 1 · deletes: 0

**What to notice**

- Advisor list has **1** item(s); client list has **1**.
- **Used Toyota Corolla** (staged add): the client sees it as a normal record; the advisor sees it flagged `create` (and the advisor's summary leaves it out until accepted).

<table>
<tr><th>Advisor GET</th><th>Client GET</th></tr>
<tr>
<td valign="top">

```json
{
  "items": [
    {
      "liability_type": "auto_loan",
      "name": "Used Toyota Corolla",
      "lender": "Local Credit Union",
      "account_last4": null,
      "owner": "client",
      "original_balance": "16500",
      "current_balance": "16500.00",
      "interest_rate": "6.4",
      "rate_type": "fixed",
      "payment_amount": "325.00",
      "payment_frequency": "monthly",
      "origination_date": "2026-11-01",
      "maturity_date": "2031-11-01",
      "is_tax_deductible": false,
      "notes": null,
      "id": "6b00ef9a7c1b5f710ea8c519",
      "client_id": "client-single-add",
      "created_at": "2026-10-22T01:10:50.228000Z",
      "updated_at": "2026-10-22T01:10:50.228000Z",
      "created_by": {
        "user_id": "cli-client-single-add",
        "caller_type": "client",
        "at": "2026-10-22T01:10:50.228000Z"
      },
      "updated_by": {
        "user_id": "cli-client-single-add",
        "caller_type": "client",
        "at": "2026-10-22T01:10:50.228000Z"
      },
      "pending_change": {
        "action": "create",
        "changes": {},
        "created_by": {
          "user_id": "cli-client-single-add",
          "caller_type": "client",
          "at": "2026-11-18T01:10:50.228000Z"
        },
        "updated_by": {
          "user_id": "cli-client-single-add",
          "caller_type": "client",
          "at": "2026-11-20T23:10:50.228000Z"
        }
      }
    }
  ],
  "total": 1
}
```

</td>
<td valign="top">

```json
{
  "items": [
    {
      "liability_type": "auto_loan",
      "name": "Used Toyota Corolla",
      "lender": "Local Credit Union",
      "account_last4": null,
      "owner": "client",
      "original_balance": "16500",
      "current_balance": "16500.00",
      "interest_rate": "6.4",
      "rate_type": "fixed",
      "payment_amount": "325.00",
      "payment_frequency": "monthly",
      "origination_date": "2026-11-01",
      "maturity_date": "2031-11-01",
      "is_tax_deductible": false,
      "notes": null,
      "id": "6b00ef9a7c1b5f710ea8c519",
      "client_id": "client-single-add",
      "created_at": "2026-10-22T01:10:50.228000Z",
      "updated_at": "2026-10-22T01:10:50.228000Z",
      "created_by": {
        "user_id": "cli-client-single-add",
        "caller_type": "client",
        "at": "2026-10-22T01:10:50.228000Z"
      },
      "updated_by": {
        "user_id": "cli-client-single-add",
        "caller_type": "client",
        "at": "2026-10-22T01:10:50.228000Z"
      }
    }
  ],
  "total": 1
}
```

</td>
</tr>
</table>

## Staged deletes only — `client-deletes-only`

Two untouched liabilities and one the client asked to delete. The client list omits it, while the advisor list shows it flagged.

Liabilities: 3 · unchanged: 2 · staged edits: 0 · adds: 0 · deletes: 1

**What to notice**

- Advisor list has **3** item(s); client list has **2**.
- **Furniture financing** (staged delete): advisor still sees the committed record flagged `delete`; the client no longer sees it.

<table>
<tr><th>Advisor GET</th><th>Client GET</th></tr>
<tr>
<td valign="top">

```json
{
  "items": [
    {
      "liability_type": "personal_loan",
      "name": "Furniture financing",
      "lender": "Wayfair Financing",
      "account_last4": null,
      "owner": "client",
      "original_balance": "3000",
      "current_balance": "0.00",
      "interest_rate": null,
      "rate_type": null,
      "payment_amount": "0.00",
      "payment_frequency": "monthly",
      "origination_date": null,
      "maturity_date": null,
      "is_tax_deductible": false,
      "notes": "Client says this is paid off.",
      "id": "6b00ef9a7c1b5f710ea8c51c",
      "client_id": "client-deletes-only",
      "created_at": "2026-10-24T01:10:50.228000Z",
      "updated_at": "2026-10-24T01:10:50.228000Z",
      "created_by": {
        "user_id": "cli-client-deletes-only",
        "caller_type": "client",
        "at": "2026-10-24T01:10:50.228000Z"
      },
      "updated_by": {
        "user_id": "cli-client-deletes-only",
        "caller_type": "client",
        "at": "2026-10-24T01:10:50.228000Z"
      },
      "pending_change": {
        "action": "delete",
        "changes": {},
        "created_by": {
          "user_id": "cli-client-deletes-only",
          "caller_type": "client",
          "at": "2026-11-18T01:10:50.228000Z"
        },
        "updated_by": {
          "user_id": "cli-client-deletes-only",
          "caller_type": "client",
          "at": "2026-11-20T23:10:50.228000Z"
        }
      }
    },
    {
      "liability_type": "student_loan",
      "name": "Undergrad loans",
      "lender": "Nelnet",
      "account_last4": null,
      "owner": "client",
      "original_balance": "25000",
      "current_balance": "7200.00",
      "interest_rate": "4.5",
      "rate_type": "fixed",
      "payment_amount": "260.00",
      "payment_frequency": "monthly",
      "origination_date": null,
      "maturity_date": null,
      "is_tax_deductible": false,
      "notes": null,
      "id": "6b00ef9a7c1b5f710ea8c51b",
      "client_id": "client-deletes-only",
      "created_at": "2026-10-23T01:10:50.228000Z",
      "updated_at": "2026-10-23T01:10:50.228000Z",
      "created_by": {
        "user_id": "adv-1",
        "caller_type": "advisor",
        "at": "2026-10-23T01:10:50.228000Z"
      },
      "updated_by": {
        "user_id": "adv-1",
        "caller_type": "advisor",
        "at": "2026-10-23T01:10:50.228000Z"
      },
      "pending_change": null
    },
    {
      "liability_type": "mortgage",
      "name": "Condo mortgage",
      "lender": "Chase",
      "account_last4": null,
      "owner": "client",
      "original_balance": "240000",
      "current_balance": "201000.00",
      "interest_rate": "3.9",
      "rate_type": "fixed",
      "payment_amount": "1130.00",
      "payment_frequency": "monthly",
      "origination_date": null,
      "maturity_date": null,
      "is_tax_deductible": true,
      "notes": null,
      "id": "6b00ef9a7c1b5f710ea8c51a",
      "client_id": "client-deletes-only",
      "created_at": "2026-10-22T01:10:50.228000Z",
      "updated_at": "2026-10-22T01:10:50.228000Z",
      "created_by": {
        "user_id": "adv-1",
        "caller_type": "advisor",
        "at": "2026-10-22T01:10:50.228000Z"
      },
      "updated_by": {
        "user_id": "adv-1",
        "caller_type": "advisor",
        "at": "2026-10-22T01:10:50.228000Z"
      },
      "pending_change": null
    }
  ],
  "total": 3
}
```

</td>
<td valign="top">

```json
{
  "items": [
    {
      "liability_type": "student_loan",
      "name": "Undergrad loans",
      "lender": "Nelnet",
      "account_last4": null,
      "owner": "client",
      "original_balance": "25000",
      "current_balance": "7200.00",
      "interest_rate": "4.5",
      "rate_type": "fixed",
      "payment_amount": "260.00",
      "payment_frequency": "monthly",
      "origination_date": null,
      "maturity_date": null,
      "is_tax_deductible": false,
      "notes": null,
      "id": "6b00ef9a7c1b5f710ea8c51b",
      "client_id": "client-deletes-only",
      "created_at": "2026-10-23T01:10:50.228000Z",
      "updated_at": "2026-10-23T01:10:50.228000Z",
      "created_by": {
        "user_id": "adv-1",
        "caller_type": "advisor",
        "at": "2026-10-23T01:10:50.228000Z"
      },
      "updated_by": {
        "user_id": "adv-1",
        "caller_type": "advisor",
        "at": "2026-10-23T01:10:50.228000Z"
      }
    },
    {
      "liability_type": "mortgage",
      "name": "Condo mortgage",
      "lender": "Chase",
      "account_last4": null,
      "owner": "client",
      "original_balance": "240000",
      "current_balance": "201000.00",
      "interest_rate": "3.9",
      "rate_type": "fixed",
      "payment_amount": "1130.00",
      "payment_frequency": "monthly",
      "origination_date": null,
      "maturity_date": null,
      "is_tax_deductible": true,
      "notes": null,
      "id": "6b00ef9a7c1b5f710ea8c51a",
      "client_id": "client-deletes-only",
      "created_at": "2026-10-22T01:10:50.228000Z",
      "updated_at": "2026-10-22T01:10:50.228000Z",
      "created_by": {
        "user_id": "adv-1",
        "caller_type": "advisor",
        "at": "2026-10-22T01:10:50.228000Z"
      },
      "updated_by": {
        "user_id": "adv-1",
        "caller_type": "advisor",
        "at": "2026-10-22T01:10:50.228000Z"
      }
    }
  ],
  "total": 2
}
```

</td>
</tr>
</table>

## No liabilities — `client-empty`

A client with no liabilities, to show an empty list response.

Liabilities: 0 · unchanged: 0 · staged edits: 0 · adds: 0 · deletes: 0

**What to notice**

- Advisor list has **0** item(s); client list has **0**.
- Both views are an empty list.

<table>
<tr><th>Advisor GET</th><th>Client GET</th></tr>
<tr>
<td valign="top">

```json
{
  "items": [],
  "total": 0
}
```

</td>
<td valign="top">

```json
{
  "items": [],
  "total": 0
}
```

</td>
</tr>
</table>
