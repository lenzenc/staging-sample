package com.example.liabilities.api;

public class LiabilityNotFoundException extends RuntimeException {

    private static final long serialVersionUID = 1L;

    public LiabilityNotFoundException() {
        super("Liability not found");
    }
}
