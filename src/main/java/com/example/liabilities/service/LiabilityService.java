package com.example.liabilities.service;

import static org.springframework.data.mongodb.core.query.Criteria.where;

import com.example.liabilities.api.CallerContext;
import com.example.liabilities.domain.Liability;
import com.example.liabilities.domain.LiabilityField;
import com.example.liabilities.domain.LiabilityFields;
import com.example.liabilities.domain.LiabilityType;
import com.example.liabilities.domain.PaymentFrequency;
import com.example.liabilities.domain.StagedAction;
import com.example.liabilities.domain.StagedChange;
import com.example.liabilities.domain.WireEnum;
import com.example.liabilities.dto.AdvisorLiabilityView;
import com.example.liabilities.dto.ClientLiabilityView;
import com.example.liabilities.dto.LiabilityList;
import com.example.liabilities.dto.LiabilityPatch;
import com.example.liabilities.dto.LiabilitySummary;
import com.example.liabilities.dto.LiabilityView;
import com.example.liabilities.dto.PendingChange;
import com.example.liabilities.dto.PendingChange.FieldChange;
import jakarta.validation.ConstraintViolation;
import jakarta.validation.ConstraintViolationException;
import jakarta.validation.Validator;
import java.math.BigDecimal;
import java.time.Clock;
import java.time.Instant;
import java.time.LocalDate;
import java.time.temporal.ChronoUnit;
import java.util.ArrayList;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.Objects;
import java.util.Optional;
import java.util.Set;
import org.bson.types.ObjectId;
import org.springframework.data.domain.Sort;
import org.springframework.data.mongodb.core.MongoTemplate;
import org.springframework.data.mongodb.core.query.Criteria;
import org.springframework.data.mongodb.core.query.Query;
import org.springframework.stereotype.Service;
import tools.jackson.databind.json.JsonMapper;

/**
 * Liability CRUD with the advisor/client staging rules: advisor edits apply immediately, while a client's edits, adds
 * and deletes are staged on the record until an advisor reviews them.
 */
@Service
public class LiabilityService {

    private final MongoTemplate mongo;
    private final Validator validator;
    private final JsonMapper json;
    private final Clock clock;

    public LiabilityService(MongoTemplate mongo, Validator validator, JsonMapper json, Clock clock) {
        this.mongo = mongo;
        this.validator = validator;
        this.json = json;
        this.clock = clock;
    }

    // ---- views ---------------------------------------------------------------------------------------------

    /**
     * Clients get their updated view (committed + staged values) with no change info. Advisors get the committed
     * values plus {@code pending_change} describing what is staged.
     */
    public LiabilityView toView(Liability doc, CallerContext caller) {
        if (caller.isClient()) {
            return ClientLiabilityView.of(effective(doc));
        }
        return new AdvisorLiabilityView(ClientLiabilityView.of(doc), pending(doc));
    }

    /**
     * The staged edits as typed values, keeping explicit nulls, in field declaration order (so responses are stable
     * whatever order the client sent them in). Only an "update" carries edits.
     */
    private Map<LiabilityField, Object> stagedValues(Liability doc) {
        Map<LiabilityField, Object> values = new LinkedHashMap<>();
        StagedChange staged = doc.staged();
        if (staged == null || staged.action() != StagedAction.UPDATE) {
            return values;
        }
        for (LiabilityField field : LiabilityField.values()) {
            if (staged.changes().containsKey(field.key())) {
                Object raw = staged.changes().get(field.key());
                values.put(field, raw == null ? null : json.convertValue(raw, field.type()));
            }
        }
        return values;
    }

    /** A copy of the liability with the given values laid over it. */
    private Liability overlay(Liability doc, Map<LiabilityField, Object> values) {
        Liability copy = doc.copy();
        values.forEach(copy::set);
        return copy;
    }

    /** The liability as a client sees it: committed values overlaid with staged edits. */
    private Liability effective(Liability doc) {
        Map<LiabilityField, Object> staged = stagedValues(doc);
        return staged.isEmpty() ? doc : overlay(doc, staged);
    }

    private PendingChange pending(Liability doc) {
        StagedChange staged = doc.staged();
        if (staged == null) {
            return null;
        }
        Map<LiabilityField, Object> values = stagedValues(doc);
        if (staged.action() == StagedAction.UPDATE && values.isEmpty()) {
            return null;
        }
        Map<String, FieldChange> changes = new LinkedHashMap<>();
        values.forEach((field, proposed) ->
                changes.put(field.key(), new FieldChange(toJsonForm(doc.get(field)), toJsonForm(proposed))));
        return new PendingChange(staged.action(), changes, staged.createdBy(), staged.updatedBy());
    }

    /** Decimals as strings, dates as ISO strings, enums as their wire value: how staged values are stored. */
    private static Object toJsonForm(Object value) {
        return switch (value) {
            case null -> null;
            case BigDecimal d -> d.toPlainString();
            case LocalDate d -> d.toString();
            case WireEnum e -> e.value();
            default -> value;
        };
    }

    private static Map<String, Object> toJsonForm(Map<LiabilityField, Object> values) {
        Map<String, Object> out = new LinkedHashMap<>();
        values.forEach((field, value) -> out.put(field.key(), toJsonForm(value)));
        return out;
    }

    /** Value equality where 1500.5 equals 1500.50. */
    private static boolean same(Object a, Object b) {
        if (a instanceof BigDecimal x && b instanceof BigDecimal y) {
            return x.compareTo(y) == 0;
        }
        return Objects.equals(a, b);
    }

    private void validate(Liability candidate) {
        Set<ConstraintViolation<LiabilityFields>> violations = validator.validate(candidate.toFields());
        if (!violations.isEmpty()) {
            throw new ConstraintViolationException(violations);
        }
    }

    private Instant now() {
        return Instant.now(clock).truncatedTo(ChronoUnit.MILLIS); // MongoDB keeps milliseconds
    }

    // ---- create / read -------------------------------------------------------------------------------------

    public Liability create(String clientId, LiabilityFields body, CallerContext caller) {
        Instant now = now();
        var stamp = caller.stamp(now);
        // A client's new liability is a proposal until an advisor reviews it.
        StagedChange staged =
                caller.isClient() ? new StagedChange(StagedAction.CREATE, new LinkedHashMap<>(), stamp, stamp) : null;
        return mongo.insert(Liability.create(clientId, body, now, stamp, staged));
    }

    public LiabilityList list(String clientId, LiabilityType type, int skip, int limit, CallerContext caller) {
        List<Criteria> all = new ArrayList<>();
        all.add(where("clientId").is(clientId));
        if (caller.isClient()) {
            all.add(where("staged.action").ne(StagedAction.DELETE));
        }
        if (type != null) {
            if (caller.isClient()) {
                // Filter on the type the client sees (staged value wins over committed).
                String staged = "staged.changes.liability_type";
                all.add(new Criteria()
                        .orOperator(
                                where("liabilityType").is(type).and(staged).exists(false),
                                where(staged).is(type.value())));
            } else {
                all.add(where("liabilityType").is(type));
            }
        }
        Criteria criteria = new Criteria().andOperator(all);
        long total = mongo.count(new Query(criteria), Liability.class);
        Query page = new Query(criteria)
                .with(Sort.by(Sort.Direction.DESC, "createdAt", "id"))
                .skip(skip)
                .limit(limit);
        List<LiabilityView> items = mongo.find(page, Liability.class).stream()
                .map(d -> toView(d, caller))
                .toList();
        return new LiabilityList(items, total);
    }

    /** The liability if it exists for this client and is visible to the caller. */
    public Optional<Liability> get(String clientId, String liabilityId, CallerContext caller) {
        if (!ObjectId.isValid(liabilityId)) {
            return Optional.empty();
        }
        Query query = new Query(
                where("id").is(new ObjectId(liabilityId)).and("clientId").is(clientId));
        return Optional.ofNullable(mongo.findOne(query, Liability.class)).filter(doc -> !hiddenFrom(doc, caller));
    }

    /** A client's pending delete hides the record from that client. */
    private static boolean hiddenFrom(Liability doc, CallerContext caller) {
        return caller.isClient() && doc.stagedAction() == StagedAction.DELETE;
    }

    // ---- update --------------------------------------------------------------------------------------------

    /** Advisor edits apply immediately; client edits are staged for advisor review. */
    public Optional<Liability> update(String clientId, String liabilityId, LiabilityPatch patch, CallerContext caller) {
        Optional<Liability> found = get(clientId, liabilityId, caller);
        if (found.isEmpty()) {
            return found;
        }
        Liability doc = found.get();
        if (caller.isClient() && doc.stagedAction() != StagedAction.CREATE) {
            stage(doc, patch.changes(), caller);
        } else {
            apply(doc, patch.changes(), caller);
        }
        return found;
    }

    private void stage(Liability doc, Map<LiabilityField, Object> changes, CallerContext caller) {
        Map<LiabilityField, Object> combined = new LinkedHashMap<>(stagedValues(doc));
        combined.putAll(changes);
        // Setting a field back to its committed value un-stages it.
        combined.entrySet().removeIf(e -> same(doc.get(e.getKey()), e.getValue()));
        // Validate the would-be record (e.g. date ordering); a violation becomes a 422.
        validate(overlay(doc, combined));
        storeStaged(doc, combined, caller);
        mongo.save(doc);
    }

    /** Set doc.staged from typed values; clear it when nothing is left. */
    private void storeStaged(Liability doc, Map<LiabilityField, Object> values, CallerContext caller) {
        if (values.isEmpty()) {
            doc.staged(null);
            return;
        }
        var stamp = caller.stamp(now());
        doc.staged(new StagedChange(
                StagedAction.UPDATE,
                toJsonForm(values),
                doc.staged() != null ? doc.staged().createdBy() : stamp,
                stamp));
    }

    private void apply(Liability doc, Map<LiabilityField, Object> changes, CallerContext caller) {
        // Re-validate the combined record (e.g. date ordering).
        Liability merged = overlay(doc, changes);
        validate(merged);
        changes.keySet().forEach(field -> doc.set(field, merged.get(field)));
        var stamp = caller.stamp(now());
        doc.touch(stamp.at(), stamp);

        if (doc.staged() != null && doc.staged().action() != StagedAction.UPDATE) {
            // Pending add/delete: the record changes, the flag stays (client edits to a pending add land here
            // too and just move the latest-edit stamp).
            if (caller.isClient()) {
                doc.staged(doc.staged().withUpdatedBy(stamp));
            }
            mongo.save(doc);
            return;
        }

        // The advisor's edit wins: drop staged fields it overlaps or that now match the record.
        Map<LiabilityField, Object> remaining = new LinkedHashMap<>(stagedValues(doc));
        remaining.entrySet().removeIf(e -> changes.containsKey(e.getKey()) || same(doc.get(e.getKey()), e.getValue()));
        if (!remaining.isEmpty()) {
            try {
                validate(overlay(doc, remaining));
            } catch (ConstraintViolationException e) {
                remaining.clear(); // leftovers no longer make a valid record
            }
        }
        if (doc.staged() != null) {
            // Prune only: keep the client's original stamps.
            doc.staged(
                    remaining.isEmpty()
                            ? null
                            : new StagedChange(
                                    doc.staged().action(),
                                    toJsonForm(remaining),
                                    doc.staged().createdBy(),
                                    doc.staged().updatedBy()));
        }
        mongo.save(doc);
    }

    // ---- delete --------------------------------------------------------------------------------------------

    /**
     * Advisors hard-delete. A client's delete is staged, unless the record is their own still-pending add (nothing
     * committed to review, so it just goes away).
     */
    public boolean delete(String clientId, String liabilityId, CallerContext caller) {
        Optional<Liability> found = get(clientId, liabilityId, caller);
        if (found.isEmpty()) {
            return false;
        }
        Liability doc = found.get();
        if (caller.isClient() && doc.stagedAction() != StagedAction.CREATE) {
            var stamp = caller.stamp(now());
            doc.staged(new StagedChange(StagedAction.DELETE, new LinkedHashMap<>(), stamp, stamp));
            mongo.save(doc);
        } else {
            mongo.remove(doc);
        }
        return true;
    }

    // ---- summary -------------------------------------------------------------------------------------------

    /** Clients get totals over their staged view; advisors over committed data. */
    public LiabilitySummary summary(String clientId, CallerContext caller) {
        List<Liability> raw = mongo.find(new Query(where("clientId").is(clientId)), Liability.class);
        Map<String, Integer> pendingByAction = new LinkedHashMap<>();
        for (Liability d : raw) {
            if (d.staged() != null) {
                pendingByAction.merge(d.staged().action().value(), 1, Integer::sum);
            }
        }
        List<Liability> docs = raw.stream()
                .filter(d -> caller.isClient()
                        ? d.stagedAction() != StagedAction.DELETE // pending deletes are gone, adds are in
                        : d.stagedAction() != StagedAction.CREATE) // pending adds are not committed yet
                .map(d -> caller.isClient() ? effective(d) : d)
                .toList();

        Map<String, Integer> countByType = new LinkedHashMap<>();
        Map<String, BigDecimal> balanceByType = new LinkedHashMap<>();
        BigDecimal totalBalance = BigDecimal.ZERO;
        BigDecimal annualPayments = BigDecimal.ZERO;
        for (Liability d : docs) {
            var type = ((LiabilityType) d.get(LiabilityField.LIABILITY_TYPE)).value();
            var balance = (BigDecimal) d.get(LiabilityField.CURRENT_BALANCE);
            countByType.merge(type, 1, Integer::sum);
            balanceByType.merge(type, balance, BigDecimal::add);
            totalBalance = totalBalance.add(balance);
            var payment = (BigDecimal) d.get(LiabilityField.PAYMENT_AMOUNT);
            var frequency = (PaymentFrequency) d.get(LiabilityField.PAYMENT_FREQUENCY);
            if (payment != null && frequency != null) {
                annualPayments = annualPayments.add(payment.multiply(BigDecimal.valueOf(frequency.paymentsPerYear())));
            }
        }
        // Clients get no change info.
        boolean advisor = !caller.isClient();
        return new LiabilitySummary(
                clientId,
                docs.size(),
                totalBalance,
                countByType,
                balanceByType,
                annualPayments,
                advisor
                        ? pendingByAction.values().stream()
                                .mapToInt(Integer::intValue)
                                .sum()
                        : 0,
                advisor ? pendingByAction : Map.of());
    }
}
