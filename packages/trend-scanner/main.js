// trend-scanner/main.js
// Fetches trending topics from Hacker News, analyzes with LLM,
// and outputs structured JSON for chaining to content-drafter.

const ctx = JSON.parse(__context__);
const config = ctx.config || {};
const chatId = config.chat_id;
const keywords = (config.keywords || "AI,startup,open-source")
  .split(",")
  .map((k) => k.trim().toLowerCase())
  .filter((k) => k.length > 0);

// Fetch top HN stories
let stories = [];
try {
  const raw = await Http.get(
    "https://hn.algolia.com/api/v1/search?tags=front_page&hitsPerPage=30"
  );
  const data = JSON.parse(raw);
  stories = (data.hits || []).map((h) => ({
    title: h.title || "",
    url: h.url || "",
    points: h.points || 0,
    comments: h.num_comments || 0,
  }));
} catch (e) {
  return `Error fetching HN: ${e}`;
}

if (stories.length === 0) {
  return "No stories found.";
}

// Count keyword matches
const keywordCounts = {};
for (const kw of keywords) {
  keywordCounts[kw] = 0;
}
for (const story of stories) {
  const titleLower = story.title.toLowerCase();
  for (const kw of keywords) {
    if (titleLower.includes(kw)) {
      keywordCounts[kw]++;
    }
  }
}

// Build story list for LLM
const storyList = stories
  .slice(0, 20)
  .map((s, i) => `${i + 1}. ${s.title} (${s.points}pts, ${s.comments}comments)`)
  .join("\n");

// Ask LLM to identify trends
const prompt = `Analyze these top Hacker News stories and identify the top 5 trending topics/themes.
For each topic, provide: topic name, 1-sentence summary, and which stories relate to it.

Stories:
${storyList}

User's interest keywords: ${keywords.join(", ")}

Respond in JSON format:
{"trends": [{"topic": "...", "summary": "...", "related_stories": [1, 3, 5]}]}`;

let trends = [];
try {
  const llmRaw = await Llm.generate(prompt);
  const llmData = JSON.parse(llmRaw);
  const text = llmData.text || "";
  // Extract JSON from LLM response
  const jsonMatch = text.match(/\{[\s\S]*\}/);
  if (jsonMatch) {
    const parsed = JSON.parse(jsonMatch[0]);
    trends = parsed.trends || [];
  }
} catch (e) {
  trends = [{ topic: "Parse Error", summary: `LLM analysis failed: ${e}`, related_stories: [] }];
}

// Store for chain consumers and standalone access
const result = {
  date: new Date().toISOString().split("T")[0],
  source: "Hacker News",
  story_count: stories.length,
  keyword_matches: keywordCounts,
  trends: trends,
};
await Storage.set("trend_scanner_latest", JSON.stringify(result));

// Send report to Telegram
const trendLines = trends
  .map((t, i) => `${i + 1}. *${t.topic}*\n   ${t.summary}`)
  .join("\n\n");

const kwLines = Object.entries(keywordCounts)
  .filter(([, count]) => count > 0)
  .map(([kw, count]) => `${kw}: ${count}건`)
  .join(", ");

const message = [
  `📊 *트렌드 리포트* (${result.date})`,
  ``,
  `*Top ${stories.length} HN 스토리 분석*`,
  ``,
  trendLines || "(트렌드 분석 없음)",
  ``,
  kwLines ? `*키워드 매칭:* ${kwLines}` : "",
  ``,
  `_Powered by KittyPaw_`,
]
  .filter((l) => l !== "")
  .join("\n");

await Telegram.sendMessage(chatId, message);

// Return structured output for chain (content-drafter receives this as prev_output)
return JSON.stringify(result);
