package com.example.liabilities.domain;

import java.time.Instant;

public record AuditStamp(String userId, CallerType callerType, Instant at) {}
