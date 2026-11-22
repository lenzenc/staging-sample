package com.example.liabilities.domain;

import java.math.BigDecimal;
import java.time.Instant;
import java.time.LocalDate;
import org.bson.types.ObjectId;
import org.springframework.data.annotation.Id;
import org.springframework.data.mongodb.core.index.CompoundIndex;
import org.springframework.data.mongodb.core.mapping.Document;
import org.springframework.data.mongodb.core.mapping.Field;

/**
 * The persisted liability. Mongo field names are the snake_case of these names (see the field naming strategy in
 * application.yaml), so documents written by the earlier Python implementation remain readable.
 */
@Document("liabilities")
// Unnamed, so MongoDB's default names (client_id_1, client_id_1_liability_type_1) match the indexes the Python
// implementation created.
@CompoundIndex(def = "{'client_id': 1}")
@CompoundIndex(def = "{'client_id': 1, 'liability_type': 1}")
public class Liability {

    @Id
    private ObjectId id;

    private String clientId;

    private LiabilityType liabilityType;
    private String name;
    private String lender;

    @Field("account_last4")
    private String accountLast4;

    private Ownership owner;
    private BigDecimal originalBalance;
    private BigDecimal currentBalance;
    private BigDecimal interestRate;
    private RateType rateType;
    private BigDecimal paymentAmount;
    private PaymentFrequency paymentFrequency;
    private LocalDate originationDate;
    private LocalDate maturityDate;
    private boolean isTaxDeductible;
    private String notes;

    private Instant createdAt;
    private Instant updatedAt;
    private AuditStamp createdBy;
    private AuditStamp updatedBy;
    private StagedChange staged;

    /** For Spring Data. */
    protected Liability() {}

    public static Liability create(
            String clientId, LiabilityFields fields, Instant at, AuditStamp stamp, StagedChange staged) {
        Liability doc = new Liability();
        doc.clientId = clientId;
        for (LiabilityField field : LiabilityField.values()) {
            doc.set(field, fields.get(field));
        }
        doc.createdAt = at;
        doc.updatedAt = at;
        doc.createdBy = stamp;
        doc.updatedBy = stamp;
        doc.staged = staged;
        return doc;
    }

    /** An independent copy, used to overlay edits without touching the stored record. */
    public Liability copy() {
        Liability c = new Liability();
        c.id = id;
        c.clientId = clientId;
        for (LiabilityField field : LiabilityField.values()) {
            c.set(field, get(field));
        }
        c.createdAt = createdAt;
        c.updatedAt = updatedAt;
        c.createdBy = createdBy;
        c.updatedBy = updatedBy;
        c.staged = staged;
        return c;
    }

    public LiabilityFields toFields() {
        return new LiabilityFields(
                liabilityType,
                name,
                lender,
                accountLast4,
                owner,
                originalBalance,
                currentBalance,
                interestRate,
                rateType,
                paymentAmount,
                paymentFrequency,
                originationDate,
                maturityDate,
                isTaxDeductible,
                notes);
    }

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

    public void set(LiabilityField field, Object value) {
        switch (field) {
            case LIABILITY_TYPE -> liabilityType = (LiabilityType) value;
            case NAME -> name = (String) value;
            case LENDER -> lender = (String) value;
            case ACCOUNT_LAST4 -> accountLast4 = (String) value;
            case OWNER -> owner = (Ownership) value;
            case ORIGINAL_BALANCE -> originalBalance = (BigDecimal) value;
            case CURRENT_BALANCE -> currentBalance = (BigDecimal) value;
            case INTEREST_RATE -> interestRate = (BigDecimal) value;
            case RATE_TYPE -> rateType = (RateType) value;
            case PAYMENT_AMOUNT -> paymentAmount = (BigDecimal) value;
            case PAYMENT_FREQUENCY -> paymentFrequency = (PaymentFrequency) value;
            case ORIGINATION_DATE -> originationDate = (LocalDate) value;
            case MATURITY_DATE -> maturityDate = (LocalDate) value;
            case IS_TAX_DEDUCTIBLE -> isTaxDeductible = (Boolean) value;
            case NOTES -> notes = (String) value;
        }
    }

    public ObjectId id() {
        return id;
    }

    public String clientId() {
        return clientId;
    }

    public Instant createdAt() {
        return createdAt;
    }

    public Instant updatedAt() {
        return updatedAt;
    }

    public AuditStamp createdBy() {
        return createdBy;
    }

    public AuditStamp updatedBy() {
        return updatedBy;
    }

    public StagedChange staged() {
        return staged;
    }

    public StagedAction stagedAction() {
        return staged == null ? null : staged.action();
    }

    public void touch(Instant at, AuditStamp by) {
        this.updatedAt = at;
        this.updatedBy = by;
    }

    public void staged(StagedChange staged) {
        this.staged = staged;
    }
}
