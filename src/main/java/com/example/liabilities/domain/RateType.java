package com.example.liabilities.domain;

import com.fasterxml.jackson.annotation.JsonValue;

public enum RateType implements WireEnum {
    FIXED("fixed"),
    VARIABLE("variable");

    private final String value;

    RateType(String value) {
        this.value = value;
    }

    @Override
    @JsonValue
    public String value() {
        return value;
    }
}
