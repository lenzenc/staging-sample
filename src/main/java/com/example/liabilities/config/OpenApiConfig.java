package com.example.liabilities.config;

import com.example.liabilities.api.CallerArgumentResolver;
import com.example.liabilities.api.CallerContext;
import io.swagger.v3.oas.models.OpenAPI;
import io.swagger.v3.oas.models.info.Info;
import io.swagger.v3.oas.models.media.StringSchema;
import io.swagger.v3.oas.models.parameters.HeaderParameter;
import java.util.Arrays;
import java.util.List;
import org.springdoc.core.customizers.OperationCustomizer;
import org.springdoc.core.utils.SpringDocUtils;
import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.Configuration;

@Configuration(proxyBeanMethods = false)
class OpenApiConfig {

    static {
        // Resolved from headers, so it is not a request parameter of its own.
        SpringDocUtils.getConfig().addRequestWrapperToIgnore(CallerContext.class);
    }

    @Bean
    OpenAPI openApi() {
        return new OpenAPI().info(new Info().title("Client Liabilities API").version("0.1.0"));
    }

    /** Documents the two caller headers on every operation that takes a {@link CallerContext}. */
    @Bean
    OperationCustomizer callerHeaders() {
        return (operation, handlerMethod) -> {
            boolean needsCaller = Arrays.stream(handlerMethod.getMethodParameters())
                    .anyMatch(p -> CallerContext.class.equals(p.getParameterType()));
            if (needsCaller) {
                operation.addParametersItem(new HeaderParameter()
                        .name(CallerArgumentResolver.CALLER_TYPE_HEADER)
                        .description("advisor or client app")
                        .required(true)
                        .schema(new StringSchema()._enum(List.of("advisor", "client"))));
                operation.addParametersItem(new HeaderParameter()
                        .name(CallerArgumentResolver.USER_ID_HEADER)
                        .description("Calling user's ID")
                        .required(true)
                        .schema(new StringSchema().minLength(1)));
            }
            return operation;
        };
    }
}
