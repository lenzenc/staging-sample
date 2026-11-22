package com.example.liabilities.config;

import com.example.liabilities.api.CallerArgumentResolver;
import com.example.liabilities.api.WireEnumConverterFactory;
import java.util.List;
import org.springframework.context.annotation.Configuration;
import org.springframework.format.FormatterRegistry;
import org.springframework.web.method.support.HandlerMethodArgumentResolver;
import org.springframework.web.servlet.config.annotation.ViewControllerRegistry;
import org.springframework.web.servlet.config.annotation.WebMvcConfigurer;

@Configuration(proxyBeanMethods = false)
class WebConfig implements WebMvcConfigurer {

    @Override
    public void addArgumentResolvers(List<HandlerMethodArgumentResolver> resolvers) {
        resolvers.add(new CallerArgumentResolver());
    }

    @Override
    public void addFormatters(FormatterRegistry registry) {
        registry.addConverterFactory(new WireEnumConverterFactory());
    }

    /** The browser test harness is a static page served from the same origin as the API. */
    @Override
    public void addViewControllers(ViewControllerRegistry registry) {
        registry.addViewController("/harness").setViewName("forward:/harness/index.html");
        registry.addViewController("/harness/").setViewName("forward:/harness/index.html");
    }
}
