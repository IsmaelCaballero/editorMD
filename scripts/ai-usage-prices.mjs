// Precios de la API de Anthropic (USD por millón de tokens, referencia 2026-09-25).
// Escritura de caché: 5 min = 1,25 × entrada; 1 h = 2 × entrada.
// Compartido por scripts/ai-usage.mjs y scripts/rq1-metrics.mjs.

/** Precios por millón de tokens. */
export const PRICES = {
  'claude-fable-5-1': { input: 10, output: 50, cacheRead: 0.25 },
  'claude-opus-5-5': { input: 4, output: 20, cacheRead: 0.2 },
  'claude-sonnet-5-5': { input: 2, output: 10, cacheRead: 0.2 },
  'claude-haiku-4-5': { input: 1, output: 5, cacheRead: 0.1 },
}
