package com.example.liabilities.domain;

import com.fasterxml.jackson.annotation.JsonValue;

public enum LiabilityType implements WireEnum {
    MORTGAGE("mortgage"),
    HOME_EQUITY_LOAN("home_equity_loan"),
    HELOC("heloc"),
    AUTO_LOAN("auto_loan"),
    STUDENT_LOAN("student_loan"),
    CREDIT_CARD("credit_card"),
    PERSONAL_LOAN("personal_loan"),
    MEDICAL_DEBT("medical_debt"),
    BUSINESS_LOAN("business_loan"),
    TAX_LIABILITY("tax_liability"),
    OTHER("other");

    private final String value;

    LiabilityType(String value) {
        this.value = value;
    }

    @Override
    @JsonValue
    public String value() {
        return value;
    }
}
