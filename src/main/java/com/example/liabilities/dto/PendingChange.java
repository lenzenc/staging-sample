package com.example.liabilities.dto;

import com.example.liabilities.domain.AuditStamp;
import com.example.liabilities.domain.StagedAction;
import java.util.Map;

/** What a client has staged for a liability, as current to proposed per field. */
public record PendingChange(
        StagedAction action, Map<String, FieldChange> changes, AuditStamp createdBy, AuditStamp updatedBy) {

    /** Both values are in JSON form (decimals as strings, dates as ISO strings). */
    public record FieldChange(Object current, Object proposed) {}
}
