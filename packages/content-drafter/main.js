// content-drafter/main.js
// Generates social media post drafts from trend analysis.
// Works as a chain step (receives prev_output) or standalone (reads from Storage).

const ctx = JSON.parse(__context__);
const config = ctx.config || {};
const chatId = config.chat_id;
const tone = config.tone || "professional";

// Get trend data: chain mode (prev_output) or standalone mode (Storage)
let trendData = null;
if (ctx.prev_output) {
  try {
    trendData = JSON.parse(ctx.prev_output);
  } catch (_) {
    trendData = null;
  }
}
if (!trendData) {
  try {
    const raw = await Storage.get("trend_scanner_latest");
    if (raw) {
      const parsed = JSON.parse(raw);
      trendData = typeof parsed === "string" ? JSON.parse(parsed) : parsed;
    }
  } catch (_) {}
}

if (!trendData || !trendData.trends || trendData.trends.length === 0) {
  await Telegram.sendMessage(chatId, "트렌드 데이터가 없습니다. trend-scanner를 먼저 실행하세요.");
  return "No trend data available.";
}

// Pick top 3 trends for content generation
const topTrends = trendData.trends.slice(0, 3);
const trendSummary = topTrends
  .map((t, i) => `${i + 1}. ${t.topic}: ${t.summary}`)
  .join("\n");

const prompt = `You are a content strategist. Based on these trending topics, create 3 social media post drafts.

Trending topics (from ${trendData.date || "today"}):
${trendSummary}

Tone: ${tone}

Create exactly 3 drafts:
1. Twitter/X post (max 280 chars, include 2-3 hashtags)
2. LinkedIn post (2-3 professional paragraphs)
3. Newsletter intro (3-5 sentences, engaging hook)

Respond in JSON:
{"drafts": [
  {"platform": "twitter", "text": "..."},
  {"platform": "linkedin", "text": "..."},
  {"platform": "newsletter", "text": "..."}
]}`;

let drafts = [];
try {
  const llmRaw = await Llm.generate(prompt);
  const llmData = JSON.parse(llmRaw);
  const text = llmData.text || "";
  const jsonMatch = text.match(/\{[\s\S]*\}/);
  if (jsonMatch) {
    const parsed = JSON.parse(jsonMatch[0]);
    drafts = parsed.drafts || [];
  }
} catch (e) {
  drafts = [{ platform: "error", text: `Draft generation failed: ${e}` }];
}

// Store drafts
await Storage.set("content_drafts_latest", JSON.stringify({
  date: trendData.date || new Date().toISOString().split("T")[0],
  tone: tone,
  drafts: drafts,
}));

// Format and send to Telegram
const parts = [`✍️ *콘텐츠 초안* (${tone})`, ""];

for (const draft of drafts) {
  const icon = draft.platform === "twitter" ? "🐦" :
               draft.platform === "linkedin" ? "💼" :
               draft.platform === "newsletter" ? "📧" : "📝";
  parts.push(`${icon} *${draft.platform.toUpperCase()}*`);
  parts.push("```");
  parts.push(draft.text || "(empty)");
  parts.push("```");
  parts.push("");
}

parts.push("_기반 데이터: trend-scanner | Powered by KittyPaw_");

await Telegram.sendMessage(chatId, parts.join("\n"));

return JSON.stringify({ drafts_count: drafts.length, tone });
