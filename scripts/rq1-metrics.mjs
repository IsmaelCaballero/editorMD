// Métricas por ejecución del experimento RQ.1 (docs/ai-usage/README.md §4.2).
//
// Lee las transcripciones de los subagentes (una por ejecución) y calcula, para
// cada una: llamadas, tokens, coste equivalente, órdenes ejecutadas y ciclos de
// *rework* (órdenes cargo/npm que terminaron con error y obligaron a corregir).
//
// Uso:  node scripts/rq1-metrics.mjs <runs.json> [salida.json]
// runs.json: { "task", "project", "session", "runs": [ { "label": "r1", "agent": "<id>" }, … ] }
// Transcripción de cada ejecución: ~/.claude/projects/<project>/<session>/subagents/agent-<id>.jsonl
// El modelo de cada ejecución se lee de la propia transcripción (message.model).
import { readFileSync, writeFileSync } from 'node:fs'
import { homedir } from 'node:os'
import { join } from 'node:path'
import { PRICES } from './ai-usage-prices.mjs'

const [, , runsFile, outFile] = process.argv
if (!runsFile) {
  console.error('Uso: node scripts/rq1-metrics.mjs <runs.json> [salida.json]')
  process.exit(1)
}
const { task, project, session, runs } = JSON.parse(readFileSync(runsFile, 'utf8'))
const subagents = join(homedir(), '.claude', 'projects', project, session, 'subagents')

/** Coste equivalente en USD de una llamada. */
function cost(model, u) {
  const p = PRICES[model] ?? PRICES['claude-opus-5-5']
  const w1h = u.cache_creation?.ephemeral_1h_input_tokens ?? 0
  const w5m =
    u.cache_creation?.ephemeral_5m_input_tokens ?? (u.cache_creation_input_tokens ?? 0) - w1h
  return (
    ((u.input_tokens ?? 0) * p.input +
      w5m * p.input * 1.25 +
      w1h * p.input * 2 +
      (u.cache_read_input_tokens ?? 0) * p.cacheRead +
      (u.output_tokens ?? 0) * p.output) /
    1e6
  )
}

/** Caracteres visibles por token de salida (calibrado en las sesiones principales: 2,0-2,2). */
const CHARS_PER_TOKEN = 2.1

// Una orden «comprobadora» que falla y va seguida de cambios cuenta como un ciclo de rework.
const CHECK = /\b(cargo\s+(test|clippy|fmt|build|check)|npm\s+(test|run\s+(lint|check|test)))\b/

function analyse(path) {
  const calls = new Map()
  const toolUses = new Map() // id → orden
  const results = [] // { cmd, isError }
  let first = null
  let last = null
  for (const line of readFileSync(path, 'utf8').split('\n')) {
    if (!line.trim()) continue
    let o
    try {
      o = JSON.parse(line)
    } catch {
      continue
    }
    if (o.timestamp) {
      first ??= o.timestamp
      last = o.timestamp
    }
    const content = Array.isArray(o.message?.content) ? o.message.content : []
    if (o.type === 'assistant' && o.message?.usage) {
      const prev = calls.get(o.message.id)
      let chars = prev?.chars ?? 0
      for (const c of content) {
        if (c.type === 'tool_use') {
          toolUses.set(c.id, { name: c.name, cmd: c.input?.command ?? '' })
          chars += JSON.stringify(c.input).length
        } else if (c.type === 'text') chars += c.text.length
      }
      const out = Math.max(prev?.u.output_tokens ?? 0, o.message.usage.output_tokens ?? 0)
      calls.set(o.message.id, {
        model: o.message.model,
        u: { ...o.message.usage, output_tokens: out },
        chars,
      })
    }
    if (o.type === 'user')
      for (const c of content)
        if (c.type === 'tool_result') {
          const t = toolUses.get(c.tool_use_id)
          if (!t) continue
          const text = Array.isArray(c.content)
            ? c.content.map((x) => x.text ?? '').join('')
            : String(c.content ?? '')
          const failed =
            c.is_error === true ||
            /error(\[E\d+\])?:|test result: FAILED|Diff in |warning: .* generated|exit code [1-9]/i.test(
              text,
            )
          results.push({ name: t.name, cmd: t.cmd, failed })
        }
  }
  const r = { calls: 0, input: 0, cacheWrite: 0, cacheRead: 0, output: 0, usd: 0, models: {} }
  r.outputRecorded = 0
  r.patched = 0
  for (const { model, u: raw, chars } of calls.values()) {
    // Las transcripciones de subagentes en segundo plano guardan a veces solo el uso
    // parcial del inicio del streaming: la salida se queda en unos pocos tokens aunque
    // el mensaje escriba un fichero entero. En ese caso se estima con el contenido visible
    // (≈ CHARS_PER_TOKEN caracteres por token, calibrado con la sesión principal). El
    // razonamiento interno de esos mensajes no queda registrado: es una cota inferior.
    let u = raw
    r.outputRecorded += raw.output_tokens ?? 0
    if (chars > 400 && (raw.output_tokens ?? 0) < chars / 20) {
      u = { ...raw, output_tokens: Math.round(chars / CHARS_PER_TOKEN) }
      r.patched++
    }
    r.calls++
    r.input += u.input_tokens ?? 0
    r.cacheWrite += u.cache_creation_input_tokens ?? 0
    r.cacheRead += u.cache_read_input_tokens ?? 0
    r.output += u.output_tokens ?? 0
    r.usd += cost(model, u)
    r.models[model] = (r.models[model] ?? 0) + 1
  }
  const checks = results.filter((x) => CHECK.test(x.cmd))
  r.toolCalls = results.length
  r.checks = checks.length
  r.failedChecks = checks.filter((x) => x.failed).length
  r.failedTools = results.filter((x) => x.failed).length
  r.minutes = first && last ? (new Date(last) - new Date(first)) / 60000 : 0
  r.model = Object.keys(r.models).sort((a, b) => r.models[b] - r.models[a])[0]
  return r
}

const rows = runs.map((run) => ({
  label: run.label,
  ...analyse(join(subagents, `agent-${run.agent}.jsonl`)),
}))
const fmt = (n) => Math.round(n).toLocaleString('es-ES')
console.log(`Tarea ${task}`)
console.log(
  '| Ejecución | Modelo | Llamadas | Lect. caché | Escr. caché | Salida (registrada; mensajes corregidos) | USD eq. | Órdenes | Comprobaciones (fallidas) | Minutos |',
)
console.log('|---|---|--:|--:|--:|--:|--:|--:|--:|--:|')
for (const r of rows)
  console.log(
    `| ${r.label} | ${r.model} | ${r.calls} | ${fmt(r.cacheRead)} | ${fmt(r.cacheWrite)} | ${fmt(r.output)} (${fmt(r.outputRecorded)}; ${r.patched}) | ${r.usd.toFixed(2)} | ${r.toolCalls} | ${r.checks} (${r.failedChecks}) | ${r.minutes.toFixed(0)} |`,
  )

// Media y desviación típica por modelo.
const byModel = {}
for (const r of rows) (byModel[r.model] ??= []).push(r)
const stats = (xs) => {
  const m = xs.reduce((a, b) => a + b, 0) / xs.length
  const sd =
    xs.length > 1 ? Math.sqrt(xs.reduce((a, b) => a + (b - m) ** 2, 0) / (xs.length - 1)) : 0
  return { mean: m, sd }
}
console.log(
  '\n| Modelo | n | USD eq. (media ± sd) | Llamadas (media ± sd) | Comprobaciones fallidas (media) |',
)
console.log('|---|--:|--:|--:|--:|')
const summary = {}
for (const [model, rs] of Object.entries(byModel)) {
  const usd = stats(rs.map((r) => r.usd))
  const calls = stats(rs.map((r) => r.calls))
  const failed = stats(rs.map((r) => r.failedChecks))
  summary[model] = { n: rs.length, usd, calls, failedChecks: failed }
  console.log(
    `| ${model} | ${rs.length} | ${usd.mean.toFixed(2)} ± ${usd.sd.toFixed(2)} | ${calls.mean.toFixed(1)} ± ${calls.sd.toFixed(1)} | ${failed.mean.toFixed(1)} |`,
  )
}
if (outFile) writeFileSync(outFile, JSON.stringify({ task, rows, summary }, null, 2) + '\n')
