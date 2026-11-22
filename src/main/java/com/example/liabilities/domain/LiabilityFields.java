package com.example.liabilities.domain;

import com.fasterxml.jackson.annotation.JsonIgnore;
import com.fasterxml.jackson.annotation.JsonProperty;
import jakarta.validation.constraints.AssertTrue;
import jakarta.validation.constraints.DecimalMax;
import jakarta.validation.constraints.DecimalMin;
import jakarta.validation.constraints.Digits;
import jakarta.validation.constraints.NotNull;
import jakarta.validation.constraints.Pattern;
import jakarta.validation.constraints.Size;
import java.math.BigDecimal;
import java.time.LocalDate;

/** The user-editable content of a liability. It is the create request body and the unit that gets validated. */
public record LiabilityFields(
        @NotNull LiabilityType liabilityType,
        @NotNull @Size(min = 1, max = 200) String name,
        @Size(max = 200) String lender,

        @JsonProperty("account_last4") @Pattern(regexp = "^\\d{4}$")
        String accountLast4,

        Ownership owner,
        @DecimalMin("0") @Digits(integer = 12, fraction = 2) BigDecimal originalBalance,

        @NotNull @DecimalMin("0") @Digits(integer = 12, fraction = 2)
        BigDecimal currentBalance,

        @DecimalMin("0") @DecimalMax("100") @Digits(integer = 3, fraction = 4)
        BigDecimal interestRate,

        RateType rateType,
        @DecimalMin("0") @Digits(integer = 12, fraction = 2) BigDecimal paymentAmount,
        PaymentFrequency paymentFrequency,
        LocalDate originationDate,
        LocalDate maturityDate,
        @JsonProperty("is_tax_deductible") Boolean isTaxDeductible,
        @Size(max = 2000) String notes) {

    /** Defaults for omitted fields: owner is the client, and the liability is not tax deductible. */
    public LiabilityFields {
        owner = owner == null ? Ownership.CLIENT : owner;
        isTaxDeductible = isTaxDeductible != null && isTaxDeductible;
    }

    @JsonIgnore
    @AssertTrue(message = "maturity_date must be on or after origination_date")
    public boolean isMaturityOnOrAfterOrigination() {
        return originationDate == null || maturityDate == null || !maturityDate.isBefore(originationDate);
    }

    /** The value of one field, for generic copy and comparison. */
    public Object get(LiabilityField field) {
        return switch (field) {
            case LIABILITY_TYPE -> liabilityType;
            case NAME -> name;
            case LENDER -> lender;
            case ACCOUNT_LAST4 -> accountLast4;
            case OWNER -> owner;
            case ORIGINAL_BALANCE -> originalBalance;
            case CURRENT_BALANCE -> currentBalance;
            case INTEREST_RATE -> interestRate;
            case RATE_TYPE -> rateType;
            case PAYMENT_AMOUNT -> paymentAmount;
            case PAYMENT_FREQUENCY -> paymentFrequency;
            case ORIGINATION_DATE -> originationDate;
            case MATURITY_DATE -> maturityDate;
            case IS_TAX_DEDUCTIBLE -> isTaxDeductible;
            case NOTES -> notes;
        };
    }
}
