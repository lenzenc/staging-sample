package com.example.liabilities.api;

import com.example.liabilities.dto.Scenario;
import com.example.liabilities.seed.SeedScenarios;
import io.swagger.v3.oas.annotations.tags.Tag;
import java.util.List;
import org.springframework.web.bind.annotation.GetMapping;
import org.springframework.web.bind.annotation.RequestMapping;
import org.springframework.web.bind.annotation.RestController;

/** Static metadata about the seeded sample clients, used by the browser test harness. No caller headers needed. */
@RestController
@RequestMapping("/api/v1/seed-scenarios")
@Tag(name = "seed")
class ScenarioController {

    private final SeedScenarios scenarios;

    ScenarioController(SeedScenarios scenarios) {
        this.scenarios = scenarios;
    }

    @GetMapping
    List<Scenario> list() {
        return scenarios.all().stream()
                .map(SeedScenarios.SeedScenario::toScenario)
                .toList();
    }
}
