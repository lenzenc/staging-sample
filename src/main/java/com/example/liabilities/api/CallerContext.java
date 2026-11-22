package com.example.liabilities.api;

import com.example.liabilities.domain.AuditStamp;
import com.example.liabilities.domain.CallerType;
import java.time.Instant;

/** Who is calling, taken from the X-Caller-Type and X-User-Id headers. Not authenticated: demo use only. */
public record CallerContext(CallerType type, String userId) {

    public boolean isClient() {
        return type == CallerType.CLIENT;
    }

    public AuditStamp stamp(Instant at) {
        return new AuditStamp(userId, type, at);
    }
}
