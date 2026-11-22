package com.example.liabilities;

import static org.assertj.core.api.Assertions.assertThat;

import java.util.List;
import java.util.Map;
import org.junit.jupiter.api.Test;
import tools.jackson.databind.JsonNode;

/** Client edits, adds and deletes are staged for advisor review; advisors apply immediately. */
class StagingApiTest extends ApiTestSupport {

    static final String URL = "/api/v1/clients/c1/liabilities";
    static final Map<String, Object> CARD = obj(
            "liability_type", "credit_card",
            "name", "Visa",
            "current_balance", "1500.50",
            "payment_amount", "100.00",
            "payment_frequency", "monthly");

    Resp create(Map<String, Object> body, Map<String, String> headers) {
        Resp r = post(URL, body, headers);
        assertThat(r.status()).as(r.text()).isEqualTo(201);
        return r;
    }

    String createId() {
        return create(CARD, ADVISOR).str("id");
    }

    Resp read(String id, Map<String, String> headers) {
        Resp r = get(URL + "/" + id, headers);
        assertThat(r.status()).as(r.text()).isEqualTo(200);
        return r;
    }

    int total(Map<String, String> headers) {
        return get(URL, headers).get("total").asInt();
    }

    List<String> keys(JsonNode node) {
        return node.propertyNames().stream().toList();
    }

    @Test
    void clientPatchIsStaged() {
        String id = createId();
        Resp r = patch(URL + "/" + id, obj("current_balance", "900.00"), CLIENT);
        assertThat(r.status()).isEqualTo(200);
        assertThat(r.str("current_balance")).isEqualTo("900.00"); // client sees their own edit

        Resp adv = read(id, ADVISOR);
        assertThat(adv.str("current_balance")).isEqualTo("1500.50"); // committed data unchanged
        assertThat(adv.get("updated_by").get("caller_type").asString()).isEqualTo("advisor");
        JsonNode pc = adv.get("pending_change");
        assertThat(pc.get("changes").toString())
                .isEqualTo("{\"current_balance\":{\"current\":\"1500.50\",\"proposed\":\"900.00\"}}");
        assertThat(pc.get("created_by").get("user_id").asString()).isEqualTo("cli-1");

        Resp cli = read(id, CLIENT);
        assertThat(cli.str("current_balance")).isEqualTo("900.00");
        assertThat(cli.str("name")).isEqualTo("Visa"); // merged with non-staged data
        assertThat(cli.has("pending_change")).isFalse(); // clients get only their updated view
    }

    @Test
    void noPendingChangeWhenNothingStaged() {
        String id = createId();
        assertThat(read(id, ADVISOR).get("pending_change").isNull()).isTrue();
        assertThat(read(id, CLIENT).has("pending_change")).isFalse();
    }

    @Test
    void repeatedClientPatchesMerge() {
        String id = createId();
        patch(URL + "/" + id, obj("current_balance", "900.00"), CLIENT);
        var other = withHeader(CLIENT, "X-User-Id", "cli-2");
        patch(URL + "/" + id, obj("current_balance", "800.00", "notes", "paying down"), other);
        JsonNode pc = read(id, ADVISOR).get("pending_change");
        assertThat(pc.get("changes").get("current_balance").get("proposed").asString())
                .isEqualTo("800.00");
        JsonNode notes = pc.get("changes").get("notes");
        assertThat(notes.get("current").isNull()).isTrue();
        assertThat(notes.get("proposed").asString()).isEqualTo("paying down");
        assertThat(pc.get("created_by").get("user_id").asString()).isEqualTo("cli-1");
        assertThat(pc.get("updated_by").get("user_id").asString()).isEqualTo("cli-2");
    }

    @Test
    void clientCanStageExplicitNull() {
        String id = create(with(CARD, "notes", "old"), ADVISOR).str("id");
        patch(URL + "/" + id, obj("notes", null), CLIENT);
        assertThat(read(id, CLIENT).get("notes").isNull()).isTrue();
        assertThat(read(id, ADVISOR).str("notes")).isEqualTo("old");
    }

    @Test
    void revertingToCommittedValueClearsStaging() {
        String id = createId();
        patch(URL + "/" + id, obj("current_balance", "900.00"), CLIENT);
        patch(URL + "/" + id, obj("current_balance", "1500.5"), CLIENT);
        assertThat(read(id, ADVISOR).get("pending_change").isNull()).isTrue();
    }

    @Test
    void clientPatchValidatesMergedRecord() {
        String id =
                create(with(CARD, "origination_date", "2020-01-01"), ADVISOR).str("id");
        Resp r = patch(URL + "/" + id, obj("maturity_date", "2019-01-01"), CLIENT);
        assertThat(r.status()).isEqualTo(422);
        assertThat(read(id, ADVISOR).get("pending_change").isNull()).isTrue();
    }

    @Test
    void advisorPatchAppliesAndDropsOverlappingStagedFields() {
        String id = createId();
        patch(URL + "/" + id, obj("current_balance", "900.00", "notes", "hi"), CLIENT);
        Resp r = patch(URL + "/" + id, obj("current_balance", "1000.00"), ADVISOR);
        assertThat(r.status()).isEqualTo(200);
        assertThat(r.str("current_balance")).isEqualTo("1000.00");
        assertThat(r.get("updated_by").get("caller_type").asString()).isEqualTo("advisor");
        JsonNode pc = r.get("pending_change");
        assertThat(keys(pc.get("changes"))).containsExactly("notes"); // overlap dropped, the rest kept
        assertThat(pc.get("updated_by").get("user_id").asString()).isEqualTo("cli-1"); // prune keeps client stamps
        assertThat(read(id, CLIENT).str("current_balance")).isEqualTo("1000.00");

        patch(URL + "/" + id, obj("notes", "advisor note"), ADVISOR);
        assertThat(read(id, ADVISOR).get("pending_change").isNull()).isTrue();
    }

    @Test
    void clientPostIsAPendingCreate() {
        createId(); // committed, 1500.50
        Resp created = create(with(CARD, "name", "New card", "current_balance", "500.00"), CLIENT);
        assertThat(created.has("pending_change")).isFalse(); // the client just sees their new record
        Resp adv = read(created.str("id"), ADVISOR);
        assertThat(adv.get("pending_change").get("action").asString()).isEqualTo("create");
        assertThat(adv.str("name")).isEqualTo("New card");
        assertThat(total(ADVISOR)).isEqualTo(2);
        // Not committed yet: the advisor's totals leave it out, the client's include it.
        Resp a = get(URL + "/summary", ADVISOR);
        Resp c = get(URL + "/summary", CLIENT);
        assertThat(a.str("total_current_balance")).isEqualTo("1500.50");
        assertThat(a.get("count").asInt()).isEqualTo(1);
        assertThat(c.str("total_current_balance")).isEqualTo("2000.50");
        assertThat(c.get("count").asInt()).isEqualTo(2);
        assertThat(a.get("pending_by_action").toString()).isEqualTo("{\"create\":1}");
        assertThat(c.get("pending_by_action").toString()).isEqualTo("{}");
        assertThat(c.get("pending_change_count").asInt()).isZero();
    }

    @Test
    void clientPatchOnPendingCreateEditsRecord() {
        Resp created = create(CARD, CLIENT);
        Resp r = patch(URL + "/" + created.str("id"), obj("current_balance", "42.00"), CLIENT);
        assertThat(r.status()).isEqualTo(200);
        Resp adv = read(created.str("id"), ADVISOR);
        assertThat(adv.str("current_balance")).isEqualTo("42.00"); // the record itself is the proposal
        assertThat(adv.get("pending_change").get("action").asString()).isEqualTo("create");
        assertThat(adv.get("pending_change").get("changes").toString()).isEqualTo("{}");
    }

    @Test
    void clientDeleteOfPendingCreateRemovesIt() {
        String id = create(CARD, CLIENT).str("id");
        assertThat(delete(URL + "/" + id, CLIENT).status()).isEqualTo(204);
        assertThat(get(URL + "/" + id, ADVISOR).status()).isEqualTo(404);
    }

    @Test
    void clientDeleteIsStagedAndHiddenFromClient() {
        String id = createId();
        patch(URL + "/" + id, obj("notes", "x"), CLIENT); // earlier staged edit
        assertThat(delete(URL + "/" + id, CLIENT).status()).isEqualTo(204);

        // Gone for the client everywhere.
        assertThat(get(URL + "/" + id, CLIENT).status()).isEqualTo(404);
        assertThat(patch(URL + "/" + id, obj("notes", "y"), CLIENT).status()).isEqualTo(404);
        assertThat(delete(URL + "/" + id, CLIENT).status()).isEqualTo(404);
        assertThat(total(CLIENT)).isZero();
        Resp c = get(URL + "/summary", CLIENT);
        assertThat(c.get("count").asInt()).isZero();
        assertThat(c.str("total_current_balance")).isEqualTo("0");

        // The advisor still sees the committed record, flagged; staged edits were discarded.
        Resp adv = read(id, ADVISOR);
        assertThat(adv.str("current_balance")).isEqualTo("1500.50");
        assertThat(adv.get("notes").isNull()).isTrue();
        assertThat(adv.get("pending_change").get("action").asString()).isEqualTo("delete");
        assertThat(adv.get("pending_change").get("changes").toString()).isEqualTo("{}");
        Resp a = get(URL + "/summary", ADVISOR);
        assertThat(a.get("count").asInt()).isEqualTo(1);
        assertThat(a.get("pending_by_action").toString()).isEqualTo("{\"delete\":1}");
    }

    @Test
    void advisorDeleteHardDeletesPendingDelete() {
        String id = createId();
        delete(URL + "/" + id, CLIENT);
        assertThat(delete(URL + "/" + id, ADVISOR).status()).isEqualTo(204);
        assertThat(get(URL + "/" + id, ADVISOR).status()).isEqualTo(404);
    }

    @Test
    void advisorPatchKeepsCreateAndDeleteFlags() {
        String created = create(CARD, CLIENT).str("id");
        Resp r = patch(URL + "/" + created, obj("name", "Fixed"), ADVISOR);
        assertThat(r.str("name")).isEqualTo("Fixed");
        assertThat(r.get("pending_change").get("action").asString()).isEqualTo("create");

        String id = createId();
        delete(URL + "/" + id, CLIENT);
        r = patch(URL + "/" + id, obj("name", "Fixed"), ADVISOR);
        assertThat(r.str("name")).isEqualTo("Fixed");
        assertThat(r.get("pending_change").get("action").asString()).isEqualTo("delete");
    }

    @Test
    void deleteRequiresCallerHeaders() {
        String id = createId();
        assertThat(delete(URL + "/" + id, Map.of()).status()).isEqualTo(422);
    }

    @Test
    void listFilterUsesViewOfCaller() {
        String id = createId();
        patch(URL + "/" + id, obj("liability_type", "personal_loan"), CLIENT);
        Map<String, String> personal = Map.of("liability_type", "personal_loan");
        Resp cli = get(URL, CLIENT, personal);
        Resp adv = get(URL, ADVISOR, personal);
        assertThat(cli.get("total").asInt()).isEqualTo(1);
        assertThat(cli.get("items").get(0).get("liability_type").asString()).isEqualTo("personal_loan");
        assertThat(adv.get("total").asInt()).isZero();
        Map<String, String> card = Map.of("liability_type", "credit_card");
        assertThat(get(URL, CLIENT, card).get("total").asInt()).isZero();
        assertThat(get(URL, ADVISOR, card).get("total").asInt()).isEqualTo(1);
    }

    @Test
    void listShowsPendingByCaller() {
        String id = createId();
        patch(URL + "/" + id, obj("current_balance", "900.00"), CLIENT);
        JsonNode adv = get(URL, ADVISOR).get("items").get(0);
        JsonNode cli = get(URL, CLIENT).get("items").get(0);
        assertThat(adv.get("current_balance").asString()).isEqualTo("1500.50");
        assertThat(adv.get("pending_change").isObject()).isTrue();
        assertThat(cli.get("current_balance").asString()).isEqualTo("900.00");
        assertThat(cli.has("pending_change")).isFalse();
    }

    @Test
    void summaryDiffersByCaller() {
        String id = createId();
        create(with(CARD, "name", "MC", "current_balance", "500.00"), ADVISOR);
        patch(URL + "/" + id, obj("current_balance", "900.00"), CLIENT);
        Resp adv = get(URL + "/summary", ADVISOR);
        Resp cli = get(URL + "/summary", CLIENT);
        assertThat(adv.str("total_current_balance")).isEqualTo("2000.50");
        assertThat(adv.get("pending_change_count").asInt()).isEqualTo(1);
        assertThat(cli.str("total_current_balance")).isEqualTo("1400.00");
        assertThat(cli.get("pending_change_count").asInt()).isZero();
    }

    @Test
    void readsRequireCallerHeaders() {
        String id = createId();
        assertThat(get(URL, Map.of()).status()).isEqualTo(422);
        assertThat(get(URL + "/" + id, Map.of()).status()).isEqualTo(422);
        assertThat(get(URL + "/summary", Map.of()).status()).isEqualTo(422);
    }

    @Test
    void clientViewHasNoChangeInfo() {
        String edited = create(with(CARD, "name", "Edited"), ADVISOR).str("id");
        create(with(CARD, "name", "Untouched"), ADVISOR);
        String gone = create(with(CARD, "name", "Gone"), ADVISOR).str("id");
        patch(URL + "/" + edited, obj("current_balance", "10.00"), CLIENT);
        create(with(CARD, "name", "Added"), CLIENT);
        delete(URL + "/" + gone, CLIENT);

        Resp cli = get(URL, CLIENT);
        assertThat(cli.get("total").asInt()).isEqualTo(3);
        Map<String, JsonNode> byName = new java.util.LinkedHashMap<>();
        cli.get("items").forEach(i -> byName.put(i.get("name").asString(), i));
        assertThat(byName.keySet()).containsExactlyInAnyOrder("Edited", "Untouched", "Added"); // delete hidden
        assertThat(byName.get("Edited").get("current_balance").asString()).isEqualTo("10.00"); // staged value wins
        assertThat(byName.values()).noneMatch(i -> i.has("pending_change"));
        assertThat(get(URL + "/summary", CLIENT).get("pending_by_action").toString())
                .isEqualTo("{}");

        Resp adv = get(URL, ADVISOR);
        assertThat(adv.get("total").asInt()).isEqualTo(4);
        adv.get("items").forEach(i -> assertThat(i.has("pending_change")).isTrue()); // null when nothing staged
    }
}
