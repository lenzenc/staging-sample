package com.example.liabilities.config;

import java.math.BigDecimal;
import java.time.Instant;
import java.time.format.DateTimeFormatter;
import java.time.format.DateTimeFormatterBuilder;
import org.springframework.boot.jackson.JacksonComponent;
import tools.jackson.core.JsonGenerator;
import tools.jackson.databind.SerializationContext;
import tools.jackson.databind.ValueSerializer;

/** Fixed JSON formats for values where the default would vary with the data. */
@JacksonComponent
public class WireFormatSerializers {

    /** Money goes over the wire as a decimal string that keeps its scale ("1500.50"), never as a JSON number. */
    public static class MoneyAsPlainString extends ValueSerializer<BigDecimal> {

        @Override
        public void serialize(BigDecimal value, JsonGenerator gen, SerializationContext ctxt) {
            gen.writeString(value.toPlainString());
        }
    }

    /** Timestamps are UTC ISO-8601 with microsecond precision ("2026-10-23T15:06:23.382000Z"), whatever the value. */
    public static class InstantWithMicros extends ValueSerializer<Instant> {

        private static final DateTimeFormatter FORMAT =
                new DateTimeFormatterBuilder().appendInstant(6).toFormatter();

        @Override
        public void serialize(Instant value, JsonGenerator gen, SerializationContext ctxt) {
            gen.writeString(FORMAT.format(value));
        }
    }
}
