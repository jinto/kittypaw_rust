// exchange-rate/main.js
// Fetches KRW exchange rates from open.er-api.com (free, no key),
// tracks changes via Storage, and alerts on target rate thresholds.

const ctx = JSON.parse(__context__);
const config = ctx.config || {};

const chatId = config.chat_id;
const currencies = (config.currencies || "USD,JPY,EUR,CNY")
  .split(",")
  .map((c) => c.trim().toUpperCase())
  .filter((c) => c.length > 0);
const alertCurrency = (config.alert_currency || "").toUpperCase();
const alertAbove = parseFloat(config.alert_above) || 0;
const alertBelow = parseFloat(config.alert_below) || 0;

// Fetch rates from open.er-api.com (free, no API key)
let data;
try {
  const raw = await Http.get("https://open.er-api.com/v6/latest/KRW");
  data = JSON.parse(raw);
} catch (e) {
  return `Error fetching exchange rates: ${e}`;
}

if (data.result !== "success") {
  return `API error: ${data["error-type"] || "unknown"}`;
}

const rates = data.rates || {};
const updateTime = data.time_last_update_utc || "";

// KRW is base=1, so 1 KRW = X foreign currency.
// We want "1 USD = ? KRW", so invert: 1/rate.
const lines = [];
const alerts = [];

for (const code of currencies) {
  const rate = rates[code];
  if (rate == null || rate === 0) {
    lines.push(`${code}: 데이터 없음`);
    continue;
  }

  // 1 foreign = how many KRW
  const krwRate = 1 / rate;

  // Load previous rate for comparison
  const prevKey = `prev_${code}`;
  const prevRaw = await Storage.get(prevKey);
  let prevRate = null;
  let delta = "";

  if (prevRaw) {
    try {
      prevRate = parseFloat(JSON.parse(prevRaw));
    } catch (_) {}
  }

  if (prevRate != null && prevRate > 0) {
    const diff = krwRate - prevRate;
    const pct = ((diff / prevRate) * 100).toFixed(2);
    const arrow = diff > 0 ? "▲" : diff < 0 ? "▼" : "−";
    delta = ` (${arrow}${Math.abs(diff).toFixed(2)}원, ${pct}%)`;
  }

  // Format based on currency magnitude
  let formatted;
  if (code === "JPY") {
    // JPY is typically shown per 100 yen
    formatted = `${code} 100¥ = ${(krwRate * 100).toFixed(2)}원${delta}`;
  } else {
    formatted = `${code} 1 = ${krwRate.toFixed(2)}원${delta}`;
  }
  lines.push(formatted);

  // Save current rate
  await Storage.set(prevKey, JSON.stringify(krwRate));

  // Check alert thresholds
  if (alertCurrency && code === alertCurrency) {
    if (alertAbove > 0 && krwRate >= alertAbove) {
      alerts.push(`⚠️ ${code} 환율이 ${krwRate.toFixed(2)}원으로 ${alertAbove}원 이상입니다!`);
    }
    if (alertBelow > 0 && krwRate <= alertBelow) {
      alerts.push(`⚠️ ${code} 환율이 ${krwRate.toFixed(2)}원으로 ${alertBelow}원 이하입니다!`);
    }
  }
}

// Build message
const parts = [`💱 *환율 현황 (KRW 기준)*`, ``];
parts.push("```");
parts.push(...lines);
parts.push("```");

if (alerts.length > 0) {
  parts.push(``);
  parts.push(`*🔔 알림*`);
  parts.push(...alerts);
}

parts.push(``);
parts.push(`_${updateTime}_`);
parts.push(`_Powered by KittyPaw_`);

const message = parts.join("\n");
await Telegram.sendMessage(chatId, message, {
  parse_mode: "Markdown",
});

const summary = alerts.length > 0
  ? `Exchange rates sent with ${alerts.length} alert(s).`
  : `Exchange rates sent for ${currencies.join(", ")}.`;

return summary;
