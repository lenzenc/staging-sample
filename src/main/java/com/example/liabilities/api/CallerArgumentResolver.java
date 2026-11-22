package com.example.liabilities.api;

import com.example.liabilities.domain.CallerType;
import org.springframework.core.MethodParameter;
import org.springframework.web.bind.support.WebDataBinderFactory;
import org.springframework.web.context.request.NativeWebRequest;
import org.springframework.web.method.support.HandlerMethodArgumentResolver;
import org.springframework.web.method.support.ModelAndViewContainer;

/** Resolves {@link CallerContext} from the caller headers; a missing or invalid header is a 422. */
public class CallerArgumentResolver implements HandlerMethodArgumentResolver {

    public static final String CALLER_TYPE_HEADER = "X-Caller-Type";
    public static final String USER_ID_HEADER = "X-User-Id";

    @Override
    public boolean supportsParameter(MethodParameter parameter) {
        return CallerContext.class.equals(parameter.getParameterType());
    }

    @Override
    public CallerContext resolveArgument(
            MethodParameter parameter,
            ModelAndViewContainer mavContainer,
            NativeWebRequest request,
            WebDataBinderFactory binderFactory) {
        String type = request.getHeader(CALLER_TYPE_HEADER);
        String userId = request.getHeader(USER_ID_HEADER);
        if (type == null || type.isBlank()) {
            throw RequestValidationException.of("header", CALLER_TYPE_HEADER, "Field required");
        }
        CallerType callerType = WireEnumConverterFactory.parse(CallerType.class, type);
        if (callerType == null) {
            throw RequestValidationException.of("header", CALLER_TYPE_HEADER, "Input should be 'advisor' or 'client'");
        }
        if (userId == null || userId.isEmpty()) {
            throw RequestValidationException.of("header", USER_ID_HEADER, "Field required");
        }
        return new CallerContext(callerType, userId);
    }
}
