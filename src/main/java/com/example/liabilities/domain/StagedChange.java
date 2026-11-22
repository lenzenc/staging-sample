package com.example.liabilities.domain;

import java.util.Map;

/**
 * A client's proposed add, edit or delete, held until an advisor reviews it.
 *
 * @param changes for updates, a sparse field-to-proposed-value map stored in JSON form (decimals as strings, dates as
 *     ISO strings, enums as their wire value); empty for create and delete. May contain explicit nulls.
 * @param createdBy the first client edit
 * @param updatedBy the latest client edit
 */
public record StagedChange(
        StagedAction action, Map<String, Object> changes, AuditStamp createdBy, AuditStamp updatedBy) {

    public StagedChange withUpdatedBy(AuditStamp stamp) {
        return new StagedChange(action, changes, createdBy, stamp);
    }
}
