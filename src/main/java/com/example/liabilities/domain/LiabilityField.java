package com.example.liabilities.domain;

import java.math.BigDecimal;
import java.time.LocalDate;
import java.util.Arrays;
import java.util.Map;
import java.util.Optional;
import java.util.function.Function;
import java.util.stream.Collectors;

/** The editable fields of a liability: the unit of PATCH bodies and of a client's staged edits. */
public enum LiabilityField {
    LIABILITY_TYPE("liability_type", LiabilityType.class, true),
    NAME("name", String.class, true),
    LENDER("lender", String.class, false),
    ACCOUNT_LAST4("account_last4", String.class, false),
    OWNER("owner", Ownership.class, true),
    ORIGINAL_BALANCE("original_balance", BigDecimal.class, false),
    CURRENT_BALANCE("current_balance", BigDecimal.class, true),
    INTEREST_RATE("interest_rate", BigDecimal.class, false),
    RATE_TYPE("rate_type", RateType.class, false),
    PAYMENT_AMOUNT("payment_amount", BigDecimal.class, false),
    PAYMENT_FREQUENCY("payment_frequency", PaymentFrequency.class, false),
    ORIGINATION_DATE("origination_date", LocalDate.class, false),
    MATURITY_DATE("maturity_date", LocalDate.class, false),
    IS_TAX_DEDUCTIBLE("is_tax_deductible", Boolean.class, true),
    NOTES("notes", String.class, false);

    private static final Map<String, LiabilityField> BY_KEY =
            Arrays.stream(values()).collect(Collectors.toUnmodifiableMap(LiabilityField::key, Function.identity()));

    private final String key;
    private final Class<?> type;
    private final boolean required;

    LiabilityField(String key, Class<?> type, boolean required) {
        this.key = key;
        this.type = type;
        this.required = required;
    }

    /** The snake_case name used in JSON bodies and in MongoDB. */
    public String key() {
        return key;
    }

    public Class<?> type() {
        return type;
    }

    /** Required fields can be changed but never cleared to null. */
    public boolean required() {
        return required;
    }

    public static Optional<LiabilityField> byKey(String key) {
        return Optional.ofNullable(BY_KEY.get(key));
    }
}
