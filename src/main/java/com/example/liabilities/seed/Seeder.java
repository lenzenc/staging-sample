package com.example.liabilities.seed;

import static org.springframework.data.mongodb.core.query.Criteria.where;

import com.example.liabilities.domain.AuditStamp;
import com.example.liabilities.domain.CallerType;
import com.example.liabilities.domain.Liability;
import com.example.liabilities.domain.LiabilityFields;
import com.example.liabilities.domain.StagedAction;
import com.example.liabilities.domain.StagedChange;
import com.example.liabilities.dto.Scenario;
import java.time.Clock;
import java.time.Duration;
import java.time.Instant;
import java.time.temporal.ChronoUnit;
import java.util.LinkedHashMap;
import java.util.Map;
import org.springframework.boot.ApplicationArguments;
import org.springframework.boot.ApplicationRunner;
import org.springframework.context.annotation.Profile;
import org.springframework.data.mongodb.core.MongoTemplate;
import org.springframework.data.mongodb.core.query.Query;
import org.springframework.stereotype.Component;
import tools.jackson.databind.JsonNode;
import tools.jackson.databind.json.JsonMapper;
import tools.jackson.databind.node.ObjectNode;

/**
 * Loads the sample client scenarios into the dev database, then exits. Re-runnable: removes existing liabilities for
 * the seeded client IDs first. Run with {@code make seed}.
 */
@Component
@Profile("seed")
class Seeder implements ApplicationRunner {

    private static final String ADVISOR_ID = "adv-1";

    private final SeedScenarios scenarios;
    private final MongoTemplate mongo;
    private final JsonMapper json;
    private final Clock clock;

    Seeder(SeedScenarios scenarios, MongoTemplate mongo, JsonMapper json, Clock clock) {
        this.scenarios = scenarios;
        this.mongo = mongo;
        this.json = json;
        this.clock = clock;
    }

    @Override
    public void run(ApplicationArguments args) {
        var clientIds = scenarios.all().stream()
                .map(SeedScenarios.SeedScenario::clientId)
                .toList();
        mongo.remove(new Query(where("clientId").in(clientIds)), Liability.class);

        Instant now = Instant.now(clock).truncatedTo(ChronoUnit.MILLIS);
        int total = 0;
        for (SeedScenarios.SeedScenario scenario : scenarios.all()) {
            int i = 0;
            for (ObjectNode item : scenario.liabilities()) {
                mongo.insert(toLiability(scenario.clientId(), item.deepCopy(), i++, now));
                total++;
            }
        }
        System.out.printf(
                "Seeded %d liabilities for %d clients:%n",
                total, scenarios.all().size());
        for (SeedScenarios.SeedScenario scenario : scenarios.all()) {
            Scenario.Counts c = scenario.counts();
            System.out.printf(
                    "  %-30s %d items, %d pending (%d edits, %d adds, %d deletes)  - %s%n",
                    scenario.clientId(),
                    c.liabilities(),
                    c.edits() + c.adds() + c.deletes(),
                    c.edits(),
                    c.adds(),
                    c.deletes(),
                    scenario.title());
        }
    }

    private Liability toLiability(String clientId, ObjectNode item, int index, Instant now) {
        JsonNode staged = item.remove("staged");
        JsonNode actionNode = item.remove("staged_action");
        boolean byClient = item.remove("by") != null;
        StagedAction action =
                actionNode == null ? StagedAction.UPDATE : json.convertValue(actionNode, StagedAction.class);
        LiabilityFields fields = json.treeToValue(item, LiabilityFields.class);

        String clientUser = "cli-" + clientId;
        Instant at = now.minus(Duration.ofDays(30 - index));
        AuditStamp stamp = byClient
                ? new AuditStamp(clientUser, CallerType.CLIENT, at)
                : new AuditStamp(ADVISOR_ID, CallerType.ADVISOR, at);

        StagedChange change = null;
        boolean hasEdits = staged != null && !staged.isEmpty();
        if (hasEdits || action != StagedAction.UPDATE) {
            // The client's pending change: first made 3 days ago, last 2 hours ago.
            Map<String, Object> changes = new LinkedHashMap<>();
            if (hasEdits) {
                staged.properties()
                        .forEach(e -> changes.put(
                                e.getKey(),
                                e.getValue().isNull() ? null : e.getValue().asString()));
            }
            change = new StagedChange(
                    action,
                    changes,
                    new AuditStamp(clientUser, CallerType.CLIENT, now.minus(Duration.ofDays(3))),
                    new AuditStamp(clientUser, CallerType.CLIENT, now.minus(Duration.ofHours(2))));
        }
        return Liability.create(clientId, fields, at, stamp, change);
    }
}
