package com.example.liabilities.api;

import io.swagger.v3.oas.annotations.tags.Tag;
import java.util.Map;
import org.bson.Document;
import org.springframework.data.mongodb.core.MongoTemplate;
import org.springframework.web.bind.annotation.GetMapping;
import org.springframework.web.bind.annotation.RestController;

@RestController
@Tag(name = "health")
class HealthController {

    private final MongoTemplate mongo;

    HealthController(MongoTemplate mongo) {
        this.mongo = mongo;
    }

    /** Pings MongoDB; an unreachable database surfaces as a 500. */
    @GetMapping("/health")
    Map<String, String> health() {
        mongo.executeCommand(new Document("ping", 1));
        return Map.of("status", "ok");
    }
}
