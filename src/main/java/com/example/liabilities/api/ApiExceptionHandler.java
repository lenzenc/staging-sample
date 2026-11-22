package com.example.liabilities.api;

import jakarta.validation.ConstraintViolation;
import jakarta.validation.ConstraintViolationException;
import java.util.ArrayList;
import java.util.List;
import java.util.Locale;
import java.util.Set;
import org.springframework.http.HttpStatus;
import org.springframework.http.ResponseEntity;
import org.springframework.http.converter.HttpMessageNotReadableException;
import org.springframework.validation.FieldError;
import org.springframework.web.bind.MethodArgumentNotValidException;
import org.springframework.web.bind.MissingServletRequestParameterException;
import org.springframework.web.bind.annotation.ExceptionHandler;
import org.springframework.web.bind.annotation.RestControllerAdvice;
import org.springframework.web.method.annotation.HandlerMethodValidationException;
import org.springframework.web.method.annotation.MethodArgumentTypeMismatchException;

/** Maps failures to the API's error shape: {@code {"detail": ...}}, with 422 for bad input and 404 for not found. */
@RestControllerAdvice
class ApiExceptionHandler {

    record Detail(Object detail) {}

    @ExceptionHandler(LiabilityNotFoundException.class)
    ResponseEntity<Detail> notFound(LiabilityNotFoundException e) {
        return ResponseEntity.status(HttpStatus.NOT_FOUND).body(new Detail(e.getMessage()));
    }

    @ExceptionHandler(RequestValidationException.class)
    ResponseEntity<Detail> invalid(RequestValidationException e) {
        return unprocessable(e.problems());
    }

    /** A merged record failed bean validation (cross-field rules and field constraints). */
    @ExceptionHandler(ConstraintViolationException.class)
    ResponseEntity<Detail> invalid(ConstraintViolationException e) {
        Set<ConstraintViolation<?>> violations = e.getConstraintViolations();
        return unprocessable(violations.stream()
                .map(v -> problem("body", v.getPropertyPath().toString(), v.getMessage()))
                .toList());
    }

    @ExceptionHandler(MethodArgumentNotValidException.class)
    ResponseEntity<Detail> invalid(MethodArgumentNotValidException e) {
        List<RequestValidationException.Problem> problems = new ArrayList<>();
        for (FieldError error : e.getBindingResult().getFieldErrors()) {
            problems.add(problem("body", error.getField(), error.getDefaultMessage()));
        }
        e.getBindingResult()
                .getGlobalErrors()
                .forEach(error -> problems.add(problem("body", error.getObjectName(), error.getDefaultMessage())));
        return unprocessable(problems);
    }

    @ExceptionHandler(HandlerMethodValidationException.class)
    ResponseEntity<Detail> invalid(HandlerMethodValidationException e) {
        List<RequestValidationException.Problem> problems = new ArrayList<>();
        e.getParameterValidationResults()
                .forEach(result -> result.getResolvableErrors()
                        .forEach(error -> problems.add(problem(
                                "query", result.getMethodParameter().getParameterName(), error.getDefaultMessage()))));
        return unprocessable(problems);
    }

    @ExceptionHandler(HttpMessageNotReadableException.class)
    ResponseEntity<Detail> unreadable(HttpMessageNotReadableException e) {
        return unprocessable(List.of(problem("body", "", "Invalid JSON or value in request body")));
    }

    @ExceptionHandler(MethodArgumentTypeMismatchException.class)
    ResponseEntity<Detail> typeMismatch(MethodArgumentTypeMismatchException e) {
        return unprocessable(List.of(problem("query", e.getName(), "Invalid value '" + e.getValue() + "'")));
    }

    @ExceptionHandler(MissingServletRequestParameterException.class)
    ResponseEntity<Detail> missingParam(MissingServletRequestParameterException e) {
        return unprocessable(List.of(problem("query", e.getParameterName(), "Field required")));
    }

    private static ResponseEntity<Detail> unprocessable(List<RequestValidationException.Problem> problems) {
        return ResponseEntity.unprocessableContent().body(new Detail(problems));
    }

    private static RequestValidationException.Problem problem(String where, String field, String msg) {
        return new RequestValidationException.Problem(List.of(where, toSnakeCase(field)), msg);
    }

    private static String toSnakeCase(String name) {
        return name.replaceAll("([a-z0-9])([A-Z])", "$1_$2").toLowerCase(Locale.ROOT);
    }
}
