package com.example.liabilities.domain;

import com.fasterxml.jackson.annotation.JsonValue;

public enum PaymentFrequency implements WireEnum {
    WEEKLY("weekly", 52),
    BIWEEKLY("biweekly", 26),
    MONTHLY("monthly", 12),
    QUARTERLY("quarterly", 4),
    ANNUALLY("annually", 1);

    private final String value;
    private final int paymentsPerYear;

    PaymentFrequency(String value, int paymentsPerYear) {
        this.value = value;
        this.paymentsPerYear = paymentsPerYear;
    }

    @Override
    @JsonValue
    public String value() {
        return value;
    }

    /** Used to annualize payments in the summary. */
    public int paymentsPerYear() {
        return paymentsPerYear;
    }
}
