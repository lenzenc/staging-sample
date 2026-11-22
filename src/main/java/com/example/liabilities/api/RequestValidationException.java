package com.example.liabilities.api;

import java.util.List;

/** A request the API rejects with 422. Carries FastAPI-style error entries (loc and msg). */
public class RequestValidationException extends RuntimeException {

    private static final long serialVersionUID = 1L;

    public record Problem(List<String> loc, String msg) {}

    private final transient List<Problem> problems;

    public RequestValidationException(List<Problem> problems) {
        super(problems.isEmpty() ? "Validation failed" : problems.getFirst().msg());
        this.problems = problems;
    }

    public static RequestValidationException of(String where, String name, String msg) {
        return new RequestValidationException(List.of(new Problem(List.of(where, name), msg)));
    }

    public List<Problem> problems() {
        return problems;
    }
}
