-- Per-call LLM token usage ledger stored as JSON array.
-- Example: [{"input_tokens":100,"output_tokens":50,"model":"claude-sonnet-4-20250514"}]
ALTER TABLE execution_history ADD COLUMN usage_json TEXT DEFAULT NULL;
