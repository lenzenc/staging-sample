package com.example.liabilities;

import static org.assertj.core.api.Assertions.assertThat;

import com.example.liabilities.seed.SeedScenarios;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import org.junit.jupiter.api.Test;
import org.springframework.beans.factory.annotation.Autowired;
import tools.jackson.databind.JsonNode;

class ScenarioApiTest extends ApiTestSupport {

    @Autowired
    SeedScenarios seedScenarios;

    @Test
    void scenariosListedWithoutHeaders() {
        Resp r = get("/api/v1/seed-scenarios", Map.of());
        assertThat(r.status()).isEqualTo(200);
        List<String> ids =
                r.json().valueStream().map(s -> s.get("client_id").asString()).toList();
        assertThat(ids)
                .isEqualTo(seedScenarios.all().stream().map(s -> s.clientId()).toList());
        assertThat(ids).hasSize(11);
        r.json().forEach(s -> {
            assertThat(s.get("title").asString()).isNotBlank();
            assertThat(s.get("description").asString()).isNotBlank();
        });
    }

    @Test
    void scenarioCounts() {
        Map<String, JsonNode> counts = new LinkedHashMap<>();
        get("/api/v1/seed-scenarios", Map.of())
                .json()
                .forEach(s -> counts.put(s.get("client_id").asString(), s.get("counts")));
        assertThat(counts.get("client-family-homeowners").toString())
                .isEqualTo("{\"liabilities\":5,\"unchanged\":1,\"edits\":2,\"adds\":1,\"deletes\":1}");
        assertThat(counts.get("client-single-add").toString())
                .isEqualTo("{\"liabilities\":1,\"unchanged\":0,\"edits\":0,\"adds\":1,\"deletes\":0}");
        assertThat(counts.get("client-no-pending").get("edits").asInt()).isZero();
        assertThat(counts.get("client-empty").get("liabilities").asInt()).isZero();
    }
}
