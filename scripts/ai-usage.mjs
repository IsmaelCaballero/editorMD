// Mide el uso REAL de tokens de una sesión de Claude Code a partir de su
// transcripción (.jsonl) y lo reparte por fase/paso del proyecto.
//
// Uso:  node scripts/ai-usage.mjs <transcripcion.jsonl> [docs/ai-usage/phases.json] [salida.json]
// Las transcripciones están en ~/.claude/projects/<proyecto>/<sesión>.jsonl
//
// Coste: precio de la API de Anthropic (USD por millón de tokens). Con una
// suscripción (Pro/Max) no se paga por token: es un «coste equivalente» que
// sirve para comparar modelos y dimensionar la licencia.
import { readFileSync, writeFileSync } from 'node:fs'

/** Precios por millón de tokens (referencia de la API, 2026-09-25). Escritura de caché a 1 h = 2 × entrada. */
export const PRICES = {
  'claude-fable-5-1': { input: 10, output: 50, cacheRead: 0.25 },
  'claude-opus-5-5': { input: 4, output: 20, cacheRead: 0.2 },
  'claude-sonnet-5-5': { input: 2, output: 10, cacheRead: 0.2 },
  'claude-haiku-4-5': { input: 1, output: 5, cacheRead: 0.1 },
}

const [, , transcript, phasesFile = 'docs/ai-usage/phases.json', outFile] = process.argv
if (!transcript) {
  console.error('Uso: node scripts/ai-usage.mjs <transcripcion.jsonl> [phases.json] [salida.json]')
  process.exit(1)
}
const { steps } = JSON.parse(readFileSync(phasesFile, 'utf8'))

// Una misma respuesta de la API aparece en varias líneas (una por bloque):
// nos quedamos con la última aparición de cada message.id.
const calls = new Map()
for (const line of readFileSync(transcript, 'utf8').split('\n')) {
  if (!line.trim()) continue
  let o
  try {
    o = JSON.parse(line)
  } catch {
    continue
  }
  if (o.type !== 'assistant' || !o.message?.usage) continue
  calls.set(o.message.id, {
    t: o.timestamp,
    model: o.message.model,
    u: o.message.usage,
    side: !!o.isSidechain,
  })
}

const empty = () => ({
  calls: 0,
  input: 0,
  cacheWrite5m: 0,
  cacheWrite1h: 0,
  cacheRead: 0,
  output: 0,
  thinking: 0,
  usd: 0,
})
const rows = steps.map((s) => ({ ...s, ...empty() }))
const models = {}

for (const { t, model, u } of calls.values()) {
  const row = rows.find((r) => t <= r.end) ?? rows.at(-1)
  const p = PRICES[model] ?? PRICES['claude-opus-5-5']
  const w1h = u.cache_creation?.ephemeral_1h_input_tokens ?? 0
  const w5m =
    u.cache_creation?.ephemeral_5m_input_tokens ?? (u.cache_creation_input_tokens ?? 0) - w1h
  const usd =
    ((u.input_tokens ?? 0) * p.input +
      w5m * p.input * 1.25 +
      w1h * p.input * 2 +
      (u.cache_read_input_tokens ?? 0) * p.cacheRead +
      (u.output_tokens ?? 0) * p.output) /
    1e6
  Object.assign(row, {
    calls: row.calls + 1,
    input: row.input + (u.input_tokens ?? 0),
    cacheWrite5m: row.cacheWrite5m + w5m,
    cacheWrite1h: row.cacheWrite1h + w1h,
    cacheRead: row.cacheRead + (u.cache_read_input_tokens ?? 0),
    output: row.output + (u.output_tokens ?? 0),
    thinking: row.thinking + (u.output_tokens_details?.thinking_tokens ?? 0),
    usd: row.usd + usd,
  })
  models[model] = (models[model] ?? 0) + 1
}

const total = rows.reduce((a, r) => {
  for (const k of Object.keys(empty())) a[k] += r[k]
  return a
}, empty())
const fmt = (n) => n.toLocaleString('es-ES')
console.log(`Modelos: ${JSON.stringify(models)} · llamadas: ${calls.size}`)
console.log(
  '| Fase | Paso | Llamadas | Entrada | Escr. caché | Lect. caché | Salida (thinking) | Coste eq. USD |',
)
console.log('|---|---|--:|--:|--:|--:|--:|--:|')
for (const r of [...rows, { phase: 'TOTAL', step: '', ...total }])
  console.log(
    `| ${r.phase} | ${r.step} | ${r.calls} | ${fmt(r.input)} | ${fmt(r.cacheWrite5m + r.cacheWrite1h)} | ${fmt(r.cacheRead)} | ${fmt(r.output)} (${fmt(r.thinking)}) | ${r.usd.toFixed(2)} |`,
  )
if (outFile)
  writeFileSync(outFile, JSON.stringify({ models, prices: PRICES, rows, total }, null, 2) + '\n')

// «¿Y si…?»: mismo número de tokens con otros modelos. Ojo: un modelo distinto
// NO produciría los mismos tokens (más o menos llamadas, más o menos rework);
// sirve para ver qué parte del coste depende del precio y cuál del volumen.
console.log('\n| Mismo uso con… | Coste eq. USD | vs. real |')
console.log('|---|--:|--:|')
for (const [model, p] of Object.entries(PRICES)) {
  const usd =
    (total.input * p.input +
      total.cacheWrite5m * p.input * 1.25 +
      total.cacheWrite1h * p.input * 2 +
      total.cacheRead * p.cacheRead +
      total.output * p.output) /
    1e6
  console.log(`| ${model} | ${usd.toFixed(2)} | ${((usd / total.usd) * 100).toFixed(0)} % |`)
}
const per = (r) => (r.calls ? Math.round(r.cacheRead / r.calls).toLocaleString('es-ES') : '—')
console.log(
  '\nContexto medio leído por llamada:',
  rows.map((r) => `${r.phase} ${r.step}: ${per(r)}`).join(' · '),
)
