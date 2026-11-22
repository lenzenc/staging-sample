package com.example.liabilities.dto;

import com.fasterxml.jackson.annotation.JsonUnwrapped;

/** The advisor's view: committed data plus what the client has staged (null when nothing is). */
public record AdvisorLiabilityView(@JsonUnwrapped ClientLiabilityView base, PendingChange pendingChange)
        implements LiabilityView {}
