import { env, SELF } from "cloudflare:test";
import { describe, it, expect, beforeEach } from "vitest";

const SECRET = "test_secret";

function webhookUrl(secret = SECRET) {
  return `http://example.com/webhook?secret=${secret}`;
}

function makePayload(overrides: Record<string, unknown> = {}) {
  return {
    action: { id: "act_001" },
    userRequest: {
      utterance: "안녕하세요",
      user: { id: "kakao_u1" },
    },
    callbackUrl: "https://callback.kakao.com/test",
    ...overrides,
  };
}

function post(url: string, body?: unknown) {
  return SELF.fetch(url, {
    method: "POST",
    ...(body ? { body: JSON.stringify(body) } : {}),
  });
}

// ── T-1: /register ──────────────────────────────────────────

describe("POST /register", () => {
  it("returns token (32 hex) and 6-digit pair code", async () => {
    const res = await post("http://example.com/register");
    expect(res.status).toBe(200);

    const json = await res.json<{ token: string; pair_code: string }>();
    expect(json.token).toMatch(/^[0-9a-f]{32}$/);
    expect(json.pair_code).toMatch(/^\d{6}$/);
  });

  it("stores tok:{token} and pair:{code} in KV", async () => {
    const res = await post("http://example.com/register");
    const { token, pair_code } = await res.json<{
      token: string;
      pair_code: string;
    }>();

    expect(await env.MESSAGES.get(`tok:${token}`)).toBe("1");
    expect(await env.MESSAGES.get(`pair:${pair_code}`)).toBe(token);
  });
});

// ── T-2: webhook pairing ────────────────────────────────────

describe("webhook pairing", () => {
  it("6-digit message creates user→token mapping", async () => {
    await env.MESSAGES.put("pair:829431", "tokenA");
    await env.MESSAGES.put("tok:tokenA", "1");

    const payload = makePayload({
      userRequest: { utterance: "829431", user: { id: "kakao_u1" } },
    });
    const res = await post(webhookUrl(), payload);
    expect(res.status).toBe(200);

    const json = await res.json<{ version: string }>();
    expect(json.version).toBe("2.0");

    // user mapping created
    expect(await env.MESSAGES.get("user:kakao_u1")).toBe("tokenA");
    // pair code consumed
    expect(await env.MESSAGES.get("pair:829431")).toBeNull();
  });

  it("invalid pair code returns error message", async () => {
    const payload = makePayload({
      userRequest: { utterance: "000000", user: { id: "kakao_u2" } },
    });
    const res = await post(webhookUrl(), payload);
    expect(res.status).toBe(200);

    const json = await res.json<{
      template: { outputs: { simpleText: { text: string } }[] };
    }>();
    expect(json.template.outputs[0].simpleText.text).toContain("유효하지 않은");
  });
});

// ── T-3: mapped user message routing ────────────────────────

describe("webhook message routing", () => {
  beforeEach(async () => {
    await env.MESSAGES.put("user:kakao_u1", "tokenA");
    await env.MESSAGES.put("tok:tokenA", "1");
  });

  it("stores message under msg:{relay_token}:{action_id}", async () => {
    const res = await post(webhookUrl(), makePayload());
    expect(res.status).toBe(200);

    const json = await res.json<{ useCallback: boolean }>();
    expect(json.useCallback).toBe(true);

    const stored = await env.MESSAGES.get("msg:tokenA:act_001");
    expect(stored).not.toBeNull();

    const msg = JSON.parse(stored!);
    expect(msg.text).toBe("안녕하세요");
    expect(msg.user_id).toBe("kakao_u1");
    expect(msg.callback_url).toBe("https://callback.kakao.com/test");
  });
});

// ── T-4: unmapped user ──────────────────────────────────────

describe("unmapped user", () => {
  it("receives connection instruction", async () => {
    const res = await post(webhookUrl(), makePayload());
    expect(res.status).toBe(200);

    const json = await res.json<{
      template: { outputs: { simpleText: { text: string } }[] };
    }>();
    expect(json.template.outputs[0].simpleText.text).toContain("연결");
  });
});

// ── T-5: poll with msg: prefix + token verification ─────────

describe("POST /poll/:token", () => {
  it("returns 401 for unregistered token", async () => {
    const res = await post("http://example.com/poll/unknown_token");
    expect(res.status).toBe(401);
  });

  it("returns 204 when no messages", async () => {
    await env.MESSAGES.put("tok:tokenA", "1");
    const res = await post("http://example.com/poll/tokenA");
    expect(res.status).toBe(204);
  });

  it("returns message from msg:{token}: prefix and deletes it", async () => {
    await env.MESSAGES.put("tok:tokenA", "1");
    await env.MESSAGES.put(
      "msg:tokenA:act1",
      JSON.stringify({
        text: "hello",
        user_id: "u1",
        callback_url: "https://cb/1",
        action_id: "act1",
      })
    );

    const res = await post("http://example.com/poll/tokenA");
    expect(res.status).toBe(200);

    const msg = await res.json<{ text: string }>();
    expect(msg.text).toBe("hello");

    // deleted after claim
    expect(await env.MESSAGES.get("msg:tokenA:act1")).toBeNull();
  });
});

// ── webhook auth ────────────────────────────────────────────

describe("webhook auth", () => {
  it("rejects missing secret with 401", async () => {
    const res = await post("http://example.com/webhook", makePayload());
    expect(res.status).toBe(401);
  });

  it("rejects wrong secret with 401", async () => {
    const res = await post(webhookUrl("wrong"), makePayload());
    expect(res.status).toBe(401);
  });
});
