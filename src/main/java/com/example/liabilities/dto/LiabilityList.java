package com.example.liabilities.dto;

import java.util.List;

public record LiabilityList(List<LiabilityView> items, long total) {}
