package com.example.liabilities.domain;

import com.fasterxml.jackson.annotation.JsonValue;

public enum Ownership implements WireEnum {
    CLIENT("client"),
    SPOUSE("spouse"),
    JOINT("joint");

    private final String value;

    Ownership(String value) {
        this.value = value;
    }

    @Override
    @JsonValue
    public String value() {
        return value;
    }
}
