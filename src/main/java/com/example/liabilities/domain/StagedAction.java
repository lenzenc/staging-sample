package com.example.liabilities.domain;

import com.fasterxml.jackson.annotation.JsonValue;

public enum StagedAction implements WireEnum {
    CREATE("create"),
    UPDATE("update"),
    DELETE("delete");

    private final String value;

    StagedAction(String value) {
        this.value = value;
    }

    @Override
    @JsonValue
    public String value() {
        return value;
    }
}
