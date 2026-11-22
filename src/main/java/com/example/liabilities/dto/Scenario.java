package com.example.liabilities.dto;

/** Static metadata about a seeded sample client, used by the browser test harness. */
public record Scenario(String clientId, String title, String description, Counts counts) {

    public record Counts(int liabilities, int unchanged, int edits, int adds, int deletes) {}
}
