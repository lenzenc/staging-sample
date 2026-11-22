package com.example.liabilities.dto;

import java.math.BigDecimal;
import java.util.Map;

public record LiabilitySummary(
        String clientId,
        int count,
        BigDecimal totalCurrentBalance,
        Map<String, Integer> countByType,
        Map<String, BigDecimal> balanceByType,
        BigDecimal totalAnnualPayments,
        int pendingChangeCount,
        Map<String, Integer> pendingByAction) {}
