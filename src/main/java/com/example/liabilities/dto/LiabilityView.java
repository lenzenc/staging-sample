package com.example.liabilities.dto;

/** What the API returns for a liability: the client's merged view, or the advisor's view with pending changes. */
public sealed interface LiabilityView permits ClientLiabilityView, AdvisorLiabilityView {}
