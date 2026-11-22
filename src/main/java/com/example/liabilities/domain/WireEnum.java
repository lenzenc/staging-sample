package com.example.liabilities.domain;

/** An enum whose JSON and MongoDB representation is a lowercase string rather than its constant name. */
public interface WireEnum {

    String value();
}
