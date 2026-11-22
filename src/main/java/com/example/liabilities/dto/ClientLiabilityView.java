package com.example.liabilities.dto;

import com.example.liabilities.domain.AuditStamp;
import com.example.liabilities.domain.Liability;
import com.example.liabilities.domain.LiabilityFields;
import com.fasterxml.jackson.annotation.JsonUnwrapped;
import java.time.Instant;

/** A liability as a client sees it: committed data merged with their staged changes, and no change information. */
public record ClientLiabilityView(
        @JsonUnwrapped LiabilityFields fields,
        String id,
        String clientId,
        Instant createdAt,
        Instant updatedAt,
        AuditStamp createdBy,
        AuditStamp updatedBy)
        implements LiabilityView {

    public static ClientLiabilityView of(Liability doc) {
        return new ClientLiabilityView(
                doc.toFields(),
                doc.id().toHexString(),
                doc.clientId(),
                doc.createdAt(),
                doc.updatedAt(),
                doc.createdBy(),
                doc.updatedBy());
    }
}
