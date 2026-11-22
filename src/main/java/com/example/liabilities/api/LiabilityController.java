package com.example.liabilities.api;

import com.example.liabilities.domain.LiabilityFields;
import com.example.liabilities.domain.LiabilityType;
import com.example.liabilities.dto.LiabilityList;
import com.example.liabilities.dto.LiabilityPatch;
import com.example.liabilities.dto.LiabilitySummary;
import com.example.liabilities.dto.LiabilityView;
import com.example.liabilities.service.LiabilityService;
import io.swagger.v3.oas.annotations.tags.Tag;
import jakarta.validation.Valid;
import jakarta.validation.constraints.Max;
import jakarta.validation.constraints.Min;
import org.springframework.http.HttpStatus;
import org.springframework.web.bind.annotation.DeleteMapping;
import org.springframework.web.bind.annotation.GetMapping;
import org.springframework.web.bind.annotation.PatchMapping;
import org.springframework.web.bind.annotation.PathVariable;
import org.springframework.web.bind.annotation.PostMapping;
import org.springframework.web.bind.annotation.RequestBody;
import org.springframework.web.bind.annotation.RequestMapping;
import org.springframework.web.bind.annotation.RequestParam;
import org.springframework.web.bind.annotation.ResponseStatus;
import org.springframework.web.bind.annotation.RestController;
import tools.jackson.databind.json.JsonMapper;
import tools.jackson.databind.node.ObjectNode;

@RestController
@RequestMapping("/api/v1/clients/{clientId}/liabilities")
@Tag(name = "liabilities")
class LiabilityController {

    private final LiabilityService service;
    private final JsonMapper json;

    LiabilityController(LiabilityService service, JsonMapper json) {
        this.service = service;
        this.json = json;
    }

    @PostMapping
    @ResponseStatus(HttpStatus.CREATED)
    LiabilityView create(
            @PathVariable String clientId, @Valid @RequestBody LiabilityFields body, CallerContext caller) {
        return service.toView(service.create(clientId, body, caller), caller);
    }

    @GetMapping
    LiabilityList list(
            @PathVariable String clientId,
            @RequestParam(name = "liability_type", required = false) LiabilityType liabilityType,
            @RequestParam(defaultValue = "0") @Min(0) int skip,
            @RequestParam(defaultValue = "50") @Min(1) @Max(200) int limit,
            CallerContext caller) {
        return service.list(clientId, liabilityType, skip, limit, caller);
    }

    @GetMapping("/summary")
    LiabilitySummary summary(@PathVariable String clientId, CallerContext caller) {
        return service.summary(clientId, caller);
    }

    @GetMapping("/{liabilityId}")
    LiabilityView get(@PathVariable String clientId, @PathVariable String liabilityId, CallerContext caller) {
        return service.get(clientId, liabilityId, caller)
                .map(doc -> service.toView(doc, caller))
                .orElseThrow(LiabilityNotFoundException::new);
    }

    /** Partial update: only the fields sent change; a client's changes are staged for advisor review. */
    @PatchMapping("/{liabilityId}")
    LiabilityView update(
            @PathVariable String clientId,
            @PathVariable String liabilityId,
            @RequestBody ObjectNode body,
            CallerContext caller) {
        LiabilityPatch patch = LiabilityPatch.parse(body, json);
        return service.update(clientId, liabilityId, patch, caller)
                .map(doc -> service.toView(doc, caller))
                .orElseThrow(LiabilityNotFoundException::new);
    }

    /** Hard delete for an advisor; staged for a client. */
    @DeleteMapping("/{liabilityId}")
    @ResponseStatus(HttpStatus.NO_CONTENT)
    void delete(@PathVariable String clientId, @PathVariable String liabilityId, CallerContext caller) {
        if (!service.delete(clientId, liabilityId, caller)) {
            throw new LiabilityNotFoundException();
        }
    }
}
