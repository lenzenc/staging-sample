package com.example.liabilities.seed;

import com.example.liabilities.dto.Scenario;
import java.io.IOException;
import java.io.InputStream;
import java.io.UncheckedIOException;
import java.util.List;
import org.springframework.core.io.ClassPathResource;
import org.springframework.stereotype.Component;
import tools.jackson.core.type.TypeReference;
import tools.jackson.databind.JsonNode;
import tools.jackson.databind.json.JsonMapper;
import tools.jackson.databind.node.ObjectNode;

/** The sample client scenarios from {@code seed/scenarios.json}: one use case per client, in display order. */
@Component
public class SeedScenarios {

    /**
     * @param liabilities liability fields plus optional seed-only keys: {@code staged} (a client's proposed field
     *     values), {@code staged_action} ("create" or "delete") and {@code by} ("client" if the client entered it)
     */
    public record SeedScenario(String clientId, String title, String description, List<ObjectNode> liabilities) {

        public Scenario toScenario() {
            return new Scenario(clientId, title, description, counts());
        }

        /** Counts of liabilities by staged kind. */
        public Scenario.Counts counts() {
            int edits = 0;
            int adds = 0;
            int deletes = 0;
            for (ObjectNode item : liabilities) {
                String action = item.path("staged_action").asString(null);
                if ("create".equals(action)) {
                    adds++;
                } else if ("delete".equals(action)) {
                    deletes++;
                } else if (action == null
                        && !item.path("staged").isMissingNode()
                        && !item.get("staged").isEmpty()) {
                    edits++;
                }
            }
            return new Scenario.Counts(
                    liabilities.size(), liabilities.size() - edits - adds - deletes, edits, adds, deletes);
        }
    }

    private final List<SeedScenario> scenarios;

    public SeedScenarios(JsonMapper json) {
        try (InputStream in = new ClassPathResource("seed/scenarios.json").getInputStream()) {
            JsonNode root = json.readTree(in);
            this.scenarios = json.convertValue(root, new TypeReference<List<SeedScenario>>() {});
        } catch (IOException e) {
            throw new UncheckedIOException(e);
        }
    }

    public List<SeedScenario> all() {
        return scenarios;
    }
}
