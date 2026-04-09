export interface Env {
  MESSAGES: KVNamespace;
  WEBHOOK_SECRET: string;
}

interface KakaoPayload {
  action: { id: string };
  userRequest: {
    utterance: string;
    user: { id: string };
  };
  callbackUrl: string;
}

interface StoredMessage {
  text: string;
  user_id: string;
  callback_url: string;
  action_id: string;
}

export default {
  async fetch(request: Request, env: Env): Promise<Response> {
    const url = new URL(request.url);

    if (request.method !== "POST") {
      return new Response("Not Found", { status: 404 });
    }

    if (url.pathname === "/register") {
      return handleRegister(env);
    }

    if (url.pathname === "/webhook") {
      return handleWebhook(request, env, url);
    }

    const pollMatch = url.pathname.match(/^\/poll\/(.+)$/);
    if (pollMatch) {
      return handlePoll(env, pollMatch[1]);
    }

    return new Response("Not Found", { status: 404 });
  },
};

async function handleRegister(env: Env): Promise<Response> {
  const token = crypto.randomUUID().replace(/-/g, "");
  const pairCode = String(Math.floor(100000 + Math.random() * 900000));

  await Promise.all([
    env.MESSAGES.put(`tok:${token}`, "1"),
    env.MESSAGES.put(`pair:${pairCode}`, token, { expirationTtl: 300 }),
  ]);

  return Response.json({ token, pair_code: pairCode });
}

async function handleWebhook(
  request: Request,
  env: Env,
  url: URL
): Promise<Response> {
  const secret = url.searchParams.get("secret");
  if (!env.WEBHOOK_SECRET || !secret || secret !== env.WEBHOOK_SECRET) {
    return new Response("Unauthorized", { status: 401 });
  }

  let payload: KakaoPayload;
  try {
    payload = JSON.parse(await request.text()) as KakaoPayload;
  } catch {
    return new Response("Bad Request", { status: 400 });
  }

  const actionId = payload.action?.id;
  const utterance = payload.userRequest?.utterance;
  const userId = payload.userRequest?.user?.id;
  const callbackUrl = payload.callbackUrl;

  if (!actionId || !utterance || !userId || !callbackUrl) {
    return new Response("Bad Request: missing required fields", { status: 400 });
  }

  // Pairing: 6-digit message → attempt to match a pair code
  if (/^\d{6}$/.test(utterance)) {
    return handlePairing(env, utterance, userId);
  }

  // Routing: look up user → relay token mapping
  const relayToken = await env.MESSAGES.get(`user:${userId}`);
  if (!relayToken) {
    return Response.json({
      version: "2.0",
      template: {
        outputs: [
          {
            simpleText: {
              text: "KittyPaw와 연결이 필요합니다. KittyPaw 앱에서 연결 코드를 확인하세요.",
            },
          },
        ],
      },
    });
  }

  const msg: StoredMessage = {
    text: utterance,
    user_id: userId,
    callback_url: callbackUrl,
    action_id: actionId,
  };

  await env.MESSAGES.put(
    `msg:${relayToken}:${actionId}`,
    JSON.stringify(msg),
    { expirationTtl: 300 }
  );

  return Response.json({ version: "2.0", useCallback: true });
}

async function handlePairing(
  env: Env,
  pairCode: string,
  kakaoUserId: string
): Promise<Response> {
  const token = await env.MESSAGES.get(`pair:${pairCode}`);
  if (!token) {
    return Response.json({
      version: "2.0",
      template: {
        outputs: [
          {
            simpleText: {
              text: "유효하지 않은 연결 코드입니다. KittyPaw 앱에서 새 코드를 확인하세요.",
            },
          },
        ],
      },
    });
  }

  await Promise.all([
    env.MESSAGES.put(`user:${kakaoUserId}`, token),
    env.MESSAGES.delete(`pair:${pairCode}`),
  ]);

  return Response.json({
    version: "2.0",
    template: {
      outputs: [{ simpleText: { text: "연결 완료!" } }],
    },
  });
}

async function handlePoll(env: Env, relayToken: string): Promise<Response> {
  const tokenExists = await env.MESSAGES.get(`tok:${relayToken}`);
  if (!tokenExists) {
    return new Response("Unauthorized", { status: 401 });
  }

  const list = await env.MESSAGES.list({
    prefix: `msg:${relayToken}:`,
    limit: 1,
  });

  if (list.keys.length === 0) {
    return new Response(null, { status: 204 });
  }

  const key = list.keys[0].name;
  const value = await env.MESSAGES.get(key);
  if (!value) {
    return new Response(null, { status: 204 });
  }

  await env.MESSAGES.delete(key);

  let parsed: unknown;
  try {
    parsed = JSON.parse(value);
  } catch {
    return new Response("Internal Error: corrupted KV value", { status: 500 });
  }
  return Response.json(parsed);
}
