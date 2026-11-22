package com.example.liabilities.dto;

import com.example.liabilities.api.RequestValidationException;
import com.example.liabilities.domain.LiabilityField;
import java.util.LinkedHashMap;
import java.util.Map;
import java.util.Optional;
import tools.jackson.core.JacksonException;
import tools.jackson.databind.JsonNode;
import tools.jackson.databind.json.JsonMapper;
import tools.jackson.databind.node.ObjectNode;

/**
 * A PATCH body: only the fields that were sent, in the order sent, with typed values. An explicit JSON null is kept
 * (it means "clear this field"), which is why this is parsed from the raw tree rather than bound to a record.
 */
public record LiabilityPatch(Map<LiabilityField, Object> changes) {

    public static LiabilityPatch parse(ObjectNode body, JsonMapper json) {
        Map<LiabilityField, Object> changes = new LinkedHashMap<>();
        for (Map.Entry<String, JsonNode> entry : body.properties()) {
            Optional<LiabilityField> field = LiabilityField.byKey(entry.getKey());
            if (field.isEmpty()) {
                continue; // unknown fields are ignored, like on create
            }
            LiabilityField f = field.get();
            JsonNode node = entry.getValue();
            if (node.isNull()) {
                if (f.required()) {
                    throw RequestValidationException.of("body", f.key(), f.key() + " cannot be null");
                }
                changes.put(f, null);
                continue;
            }
            try {
                changes.put(f, json.treeToValue(node, f.type()));
            } catch (JacksonException e) {
                throw RequestValidationException.of("body", f.key(), "invalid value for " + f.key());
            }
        }
        return new LiabilityPatch(changes);
    }
}
