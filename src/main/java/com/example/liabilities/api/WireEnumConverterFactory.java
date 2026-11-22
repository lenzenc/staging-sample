package com.example.liabilities.api;

import com.example.liabilities.domain.WireEnum;
import org.springframework.core.convert.converter.Converter;
import org.springframework.core.convert.converter.ConverterFactory;

/** Binds request params to {@link WireEnum} constants by wire value ("credit_card"), not constant name. */
public class WireEnumConverterFactory implements ConverterFactory<String, WireEnum> {

    @Override
    @SuppressWarnings({"unchecked", "rawtypes"})
    public <T extends WireEnum> Converter<String, T> getConverter(Class<T> targetType) {
        return source -> {
            T parsed = (T) parse((Class) targetType, source);
            if (parsed == null) {
                throw new IllegalArgumentException("No " + targetType.getSimpleName() + " with value '" + source + "'");
            }
            return parsed;
        };
    }

    /** The constant whose wire value matches, or null. */
    public static <E extends Enum<E> & WireEnum> E parse(Class<E> type, String value) {
        for (E constant : type.getEnumConstants()) {
            if (constant.value().equals(value)) {
                return constant;
            }
        }
        return null;
    }
}
