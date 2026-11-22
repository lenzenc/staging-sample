package com.example.liabilities;

import static org.assertj.core.api.Assertions.assertThat;

import java.util.LinkedHashMap;
import java.util.Map;
import java.util.stream.Stream;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.params.ParameterizedTest;
import org.junit.jupiter.params.provider.Arguments;
import org.junit.jupiter.params.provider.MethodSource;

class LiabilityApiTest extends ApiTestSupport {

    static final String URL = "/api/v1/clients/c1/liabilities";
    static final Map<String, Object> MORTGAGE = obj(
            "liability_type", "mortgage",
            "name", "Primary mortgage",
            "current_balance", "250000.00",
            "interest_rate", "6.5",
            "payment_amount", "2000.00",
            "payment_frequency", "monthly");
    static final Map<String, Object> CARD =
            obj("liability_type", "credit_card", "name", "Visa", "current_balance", "1500.50");

    Resp create(Map<String, Object> body) {
        Resp r = post(URL, body, ADVISOR);
        assertThat(r.status()).as(r.text()).isEqualTo(201);
        return r;
    }

    @Test
    void crudAndAudit() {
        Resp created = create(MORTGAGE);
        assertThat(created.get("created_by").get("caller_type").asString()).isEqualTo("advisor");
        String lid = created.str("id");

        var otherAdvisor = withHeader(ADVISOR, "X-User-Id", "adv-2");
        Resp r = patch(URL + "/" + lid, obj("current_balance", "240000.00"), otherAdvisor);
        assertThat(r.status()).isEqualTo(200);
        assertThat(r.str("current_balance")).isEqualTo("240000.00");
        assertThat(r.str("name")).isEqualTo("Primary mortgage"); // untouched
        assertThat(r.get("created_by").get("user_id").asString()).isEqualTo("adv-1");
        assertThat(r.get("updated_by").get("user_id").asString()).isEqualTo("adv-2");

        assertThat(get(URL + "/" + lid, ADVISOR).str("current_balance")).isEqualTo("240000.00");
        assertThat(delete(URL + "/" + lid, ADVISOR).status()).isEqualTo(204);
        assertThat(get(URL + "/" + lid, ADVISOR).status()).isEqualTo(404);
    }

    @Test
    void listFilterPagination() {
        create(MORTGAGE);
        create(CARD);
        create(CARD);
        assertThat(get(URL, ADVISOR).get("total").asInt()).isEqualTo(3);
        assertThat(get(URL, ADVISOR, Map.of("liability_type", "credit_card"))
                        .get("total")
                        .asInt())
                .isEqualTo(2);
        Resp page = get(URL, ADVISOR, Map.of("limit", "1", "skip", "1"));
        assertThat(page.get("total").asInt()).isEqualTo(3);
        assertThat(page.get("items").size()).isEqualTo(1);
    }

    @Test
    void invalidPagingIsRejected() {
        assertThat(get(URL, ADVISOR, Map.of("limit", "0")).status()).isEqualTo(422);
        assertThat(get(URL, ADVISOR, Map.of("limit", "201")).status()).isEqualTo(422);
        assertThat(get(URL, ADVISOR, Map.of("skip", "-1")).status()).isEqualTo(422);
        assertThat(get(URL, ADVISOR, Map.of("liability_type", "bogus")).status())
                .isEqualTo(422);
    }

    @Test
    void summary() {
        create(MORTGAGE);
        create(CARD);
        Resp s = get(URL + "/summary", ADVISOR);
        assertThat(s.get("count").asInt()).isEqualTo(2);
        assertThat(s.str("total_current_balance")).isEqualTo("251500.50");
        assertThat(s.get("count_by_type").toString()).isEqualTo("{\"mortgage\":1,\"credit_card\":1}");
        assertThat(s.str("total_annual_payments")).isEqualTo("24000.00");
    }

    @Test
    void clientIsolation() {
        String id = create(MORTGAGE).str("id");
        String other = "/api/v1/clients/c2/liabilities";
        assertThat(get(other + "/" + id, ADVISOR).status()).isEqualTo(404);
        assertThat(get(other, ADVISOR).get("total").asInt()).isZero();
        assertThat(delete(other + "/" + id, ADVISOR).status()).isEqualTo(404);
    }

    static Stream<Arguments> badCallerHeaders() {
        return Stream.of(
                Arguments.of(Map.<String, String>of()),
                Arguments.of(Map.of("X-Caller-Type", "advisor")),
                Arguments.of(Map.of("X-Caller-Type", "bogus", "X-User-Id", "x")));
    }

    @ParameterizedTest
    @MethodSource("badCallerHeaders")
    void badCallerHeadersAre422(Map<String, String> headers) {
        assertThat(post(URL, MORTGAGE, headers).status()).isEqualTo(422);
    }

    static Stream<Arguments> invalidFields() {
        return Stream.of(
                Arguments.of(obj("current_balance", "-1")),
                Arguments.of(obj("account_last4", "12")),
                Arguments.of(obj("interest_rate", "150")),
                Arguments.of(obj("current_balance", null)));
    }

    @ParameterizedTest
    @MethodSource("invalidFields")
    void validationErrors(Map<String, Object> patch) {
        Map<String, Object> body = new LinkedHashMap<>(MORTGAGE);
        body.putAll(patch);
        assertThat(post(URL, body, ADVISOR).status()).isEqualTo(422);
    }

    @Test
    void dateValidation() {
        var bad = with(MORTGAGE, "origination_date", "2020-01-01", "maturity_date", "2019-01-01");
        assertThat(post(URL, bad, ADVISOR).status()).isEqualTo(422);
        String id = create(with(MORTGAGE, "origination_date", "2020-01-01")).str("id");
        Resp r = patch(URL + "/" + id, obj("maturity_date", "2019-01-01"), ADVISOR);
        assertThat(r.status()).isEqualTo(422);
    }

    @Test
    void invalidIdIs404() {
        assertThat(get(URL + "/not-an-id", ADVISOR).status()).isEqualTo(404);
    }

    @Test
    void wireFormatKeepsMoneyAsScaledStringsAndOmitsNothingForAdvisors() {
        Resp r = create(CARD);
        assertThat(r.str("current_balance")).isEqualTo("1500.50");
        assertThat(r.get("lender").isNull()).isTrue();
        assertThat(r.get("owner").asString()).isEqualTo("client");
        assertThat(r.get("is_tax_deductible").asBoolean()).isFalse();
        assertThat(r.has("pending_change")).isTrue();
        assertThat(r.get("pending_change").isNull()).isTrue();
    }
}
