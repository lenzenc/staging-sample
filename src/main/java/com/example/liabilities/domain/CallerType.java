package com.example.liabilities.domain;

import com.fasterxml.jackson.annotation.JsonValue;

public enum CallerType implements WireEnum {
    ADVISOR("advisor"),
    CLIENT("client");

    private final String value;

    CallerType(String value) {
        this.value = value;
    }

    @Override
    @JsonValue
    public String value() {
        return value;
    }
}
