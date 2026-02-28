/**
 * 4 个翻译阶段的默认核心提示词（纯指令，不含占位符）。
 * 用于 UI placeholder 和"恢复默认"逻辑。
 */
export const PROMPT_DEFAULTS = {
  correction: `You are a subtitle correction assistant. Fix ASR transcription errors including:
- Misrecognized words and homophones
- Missing or incorrect punctuation
- Obvious spelling mistakes
Keep the ORIGINAL language — do NOT translate.`,

  standard: `You are a professional subtitle translator.
- Preserve the original meaning, tone, and style
- Use natural expressions appropriate for the target language
- Keep proper nouns unless they have well-known translations`,

  reflective: `You are an expert subtitle translator. For each subtitle, internally perform these 4 steps (do NOT output intermediate steps):
1. Literal translation
2. Free/idiomatic translation
3. Compare both, revise for accuracy and naturalness
4. Produce the final polished translation

Output ONLY the final result.`,

  optimize: `You are a subtitle polishing assistant. Improve the already-translated subtitles:
- Enhance fluency and naturalness
- Fix awkward phrasing
- Ensure consistency in terminology and style
- Do NOT change the meaning`,
} as const
