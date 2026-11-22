package com.example.liabilities.config;

import com.example.liabilities.domain.CallerType;
import com.example.liabilities.domain.LiabilityType;
import com.example.liabilities.domain.Ownership;
import com.example.liabilities.domain.PaymentFrequency;
import com.example.liabilities.domain.RateType;
import com.example.liabilities.domain.StagedAction;
import com.example.liabilities.domain.WireEnum;
import java.time.Clock;
import java.util.List;
import java.util.Set;
import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.Configuration;
import org.springframework.core.convert.TypeDescriptor;
import org.springframework.core.convert.converter.GenericConverter;
import org.springframework.data.convert.ReadingConverter;
import org.springframework.data.convert.WritingConverter;
import org.springframework.data.mongodb.core.convert.MongoCustomConversions;
import org.springframework.data.mongodb.core.convert.MongoCustomConversions.MongoConverterConfigurationAdapter;

@Configuration(proxyBeanMethods = false)
class MongoConfig {

    private static final List<Class<? extends Enum<?>>> WIRE_ENUMS = List.of(
            CallerType.class,
            LiabilityType.class,
            Ownership.class,
            PaymentFrequency.class,
            RateType.class,
            StagedAction.class);

    @Bean
    Clock clock() {
        return Clock.systemUTC();
    }

    /**
     * Matches the document shape the Python implementation wrote: money as Decimal128, dates as UTC dates, enums as
     * lowercase strings.
     */
    @Bean
    MongoCustomConversions mongoCustomConversions() {
        return MongoCustomConversions.create((MongoConverterConfigurationAdapter adapter) -> {
            adapter.bigDecimal(MongoCustomConversions.BigDecimalRepresentation.DECIMAL128);
            adapter.useNativeDriverJavaTimeCodecs();
            adapter.registerConverters(List.of(new WireEnumWriter(), new WireEnumReader()));
        });
    }

    @WritingConverter
    static class WireEnumWriter implements GenericConverter {

        @Override
        public Set<ConvertiblePair> getConvertibleTypes() {
            return Set.copyOf(WIRE_ENUMS.stream()
                    .map(type -> new ConvertiblePair(type, String.class))
                    .toList());
        }

        @Override
        public Object convert(Object source, TypeDescriptor sourceType, TypeDescriptor targetType) {
            return source == null ? null : ((WireEnum) source).value();
        }
    }

    @ReadingConverter
    static class WireEnumReader implements GenericConverter {

        @Override
        public Set<ConvertiblePair> getConvertibleTypes() {
            return Set.copyOf(WIRE_ENUMS.stream()
                    .map(type -> new ConvertiblePair(String.class, type))
                    .toList());
        }

        @Override
        @SuppressWarnings({"unchecked", "rawtypes"})
        public Object convert(Object source, TypeDescriptor sourceType, TypeDescriptor targetType) {
            if (source == null) {
                return null;
            }
            for (Object constant : targetType.getType().getEnumConstants()) {
                if (((WireEnum) constant).value().equals(source)) {
                    return constant;
                }
            }
            throw new IllegalArgumentException("Unknown " + targetType.getType().getSimpleName() + ": " + source);
        }
    }
}
