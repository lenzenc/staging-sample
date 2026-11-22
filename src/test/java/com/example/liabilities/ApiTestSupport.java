package com.example.liabilities;

import static org.springframework.test.web.servlet.request.MockMvcRequestBuilders.request;

import com.example.liabilities.domain.Liability;
import java.util.LinkedHashMap;
import java.util.Map;
import org.junit.jupiter.api.BeforeEach;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.boot.test.context.SpringBootTest;
import org.springframework.boot.webmvc.test.autoconfigure.AutoConfigureMockMvc;
import org.springframework.context.annotation.Import;
import org.springframework.data.mongodb.core.MongoTemplate;
import org.springframework.data.mongodb.core.query.Query;
import org.springframework.http.HttpMethod;
import org.springframework.http.MediaType;
import org.springframework.mock.web.MockHttpServletResponse;
import org.springframework.test.web.servlet.MockMvc;
import tools.jackson.databind.JsonNode;
import tools.jackson.databind.json.JsonMapper;

/** Boots the app against a real MongoDB and gives tests a small HTTP helper that returns status plus parsed JSON. */
@SpringBootTest
@AutoConfigureMockMvc
@Import(TestcontainersConfig.class)
abstract class ApiTestSupport {

    static final Map<String, String> ADVISOR = Map.of("X-Caller-Type", "advisor", "X-User-Id", "adv-1");
    static final Map<String, String> CLIENT = Map.of("X-Caller-Type", "client", "X-User-Id", "cli-1");

    /** A response: HTTP status and the body parsed as JSON (null when empty). */
    record Resp(int status, JsonNode json, String text) {

        JsonNode get(String field) {
            return json.get(field);
        }

        String str(String field) {
            JsonNode node = json.get(field);
            return node == null || node.isNull() ? null : node.asString();
        }

        boolean has(String field) {
            return json.has(field);
        }
    }

    @Autowired
    MockMvc mvc;

    @Autowired
    JsonMapper mapper;

    @Autowired
    MongoTemplate mongo;

    @BeforeEach
    void cleanDatabase() {
        mongo.remove(new Query(), Liability.class);
    }

    Resp send(HttpMethod method, String url, Object body, Map<String, String> headers, Map<String, String> params) {
        try {
            var req = request(method, url);
            headers.forEach(req::header);
            params.forEach(req::param);
            if (body != null) {
                req.contentType(MediaType.APPLICATION_JSON).content(mapper.writeValueAsString(body));
            }
            MockHttpServletResponse res = mvc.perform(req).andReturn().getResponse();
            String text = res.getContentAsString();
            return new Resp(res.getStatus(), text.isEmpty() ? null : mapper.readTree(text), text);
        } catch (Exception e) {
            throw new AssertionError(e);
        }
    }

    Resp get(String url, Map<String, String> headers) {
        return send(HttpMethod.GET, url, null, headers, Map.of());
    }

    Resp get(String url, Map<String, String> headers, Map<String, String> params) {
        return send(HttpMethod.GET, url, null, headers, params);
    }

    Resp post(String url, Object body, Map<String, String> headers) {
        return send(HttpMethod.POST, url, body, headers, Map.of());
    }

    Resp patch(String url, Object body, Map<String, String> headers) {
        return send(HttpMethod.PATCH, url, body, headers, Map.of());
    }

    Resp delete(String url, Map<String, String> headers) {
        return send(HttpMethod.DELETE, url, null, headers, Map.of());
    }

    /** A JSON object from alternating keys and values; unlike Map.of it allows null values. */
    static Map<String, Object> obj(Object... keyValues) {
        Map<String, Object> map = new LinkedHashMap<>();
        for (int i = 0; i < keyValues.length; i += 2) {
            map.put((String) keyValues[i], keyValues[i + 1]);
        }
        return map;
    }

    /** {@code base} with {@code keyValues} laid over it (the Python tests' {@code {**base, **more}}). */
    static Map<String, Object> with(Map<String, Object> base, Object... keyValues) {
        Map<String, Object> map = new LinkedHashMap<>(base);
        map.putAll(obj(keyValues));
        return map;
    }

    static Map<String, String> withHeader(Map<String, String> base, String name, String value) {
        Map<String, String> map = new LinkedHashMap<>(base);
        map.put(name, value);
        return map;
    }
}
