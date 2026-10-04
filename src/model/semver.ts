/**
 * **M17 `SemVer`** (Modelo · TS): análisis, comparación e incremento de versiones
 * según [Semantic Versioning 2.0.0](https://semver.org/lang/es/).
 *
 * Es la base para registrar las versiones de los componentes (PLAN.md §4.3)
 * y de la propia aplicación.
 *
 * @packageDocumentation
 */

/** Parte de la versión que se incrementa con {@link bump}. */
export type ReleaseType = 'major' | 'minor' | 'patch'

/**
 * Versión SemVer descompuesta en sus partes.
 *
 * @example
 * `1.0.0-rc.1+build.5` → `{ major: 1, minor: 0, patch: 0, prerelease: ['rc', 1], build: ['build', '5'] }`
 */
export interface SemVer {
  /** Se incrementa con cambios incompatibles en la API pública. */
  readonly major: number
  /** Se incrementa con funcionalidad nueva compatible. */
  readonly minor: number
  /** Se incrementa con correcciones compatibles. */
  readonly patch: number
  /**
   * Identificadores de pre-release (tras `-`). Los numéricos se guardan como
   * `number` porque se comparan numéricamente. Vacío = versión final.
   */
  readonly prerelease: readonly (string | number)[]
  /** Metadatos de compilación (tras `+`). **No** afectan a la precedencia. */
  readonly build: readonly string[]
}

/**
 * Expresión regular oficial de semver.org (con grupos con nombre), anclada al
 * principio y al final.
 */
const SEMVER_RE =
  /^(?<major>0|[1-9]\d*)\.(?<minor>0|[1-9]\d*)\.(?<patch>0|[1-9]\d*)(?:-(?<pre>(?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*)(?:\.(?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*))*))?(?:\+(?<build>[0-9a-zA-Z-]+(?:\.[0-9a-zA-Z-]+)*))?$/

/**
 * Convierte un texto en {@link SemVer}.
 *
 * @param text - Versión, p. ej. `"1.2.3-beta.1+exp.sha.5114f85"`. No admite el prefijo `v`.
 * @returns La versión descompuesta.
 * @throws `Error` si el texto no cumple SemVer 2.0.0 (p. ej. `"1.2"`, `"01.2.3"`, `"1.2.3-"`).
 *
 * @example
 * ```ts
 * parse('2.1.0-alpha.3').prerelease // ['alpha', 3]
 * ```
 */
export function parse(text: string): SemVer {
  const m = SEMVER_RE.exec(text)
  if (!m?.groups) {
    throw new Error(`Versión SemVer no válida: "${text}"`)
  }
  const g = m.groups
  return {
    major: Number(g.major),
    minor: Number(g.minor),
    patch: Number(g.patch),
    prerelease: g.pre ? g.pre.split('.').map((id) => (/^\d+$/.test(id) ? Number(id) : id)) : [],
    build: g.build ? g.build.split('.') : [],
  }
}

/**
 * Indica si un texto es una versión SemVer 2.0.0 válida.
 *
 * @param text - Texto a comprobar.
 * @returns `true` si {@link parse} lo aceptaría.
 */
export function isValid(text: string): boolean {
  return SEMVER_RE.test(text)
}

/**
 * Vuelve a convertir una {@link SemVer} en texto canónico.
 *
 * @param v - Versión.
 * @returns Texto `MAJOR.MINOR.PATCH[-PRE][+BUILD]`.
 */
export function format(v: SemVer): string {
  let s = `${v.major}.${v.minor}.${v.patch}`
  if (v.prerelease.length) s += `-${v.prerelease.join('.')}`
  if (v.build.length) s += `+${v.build.join('.')}`
  return s
}

/**
 * Compara dos identificadores de pre-release (regla 11.4 de la especificación).
 * Numérico < alfanumérico; dos numéricos se comparan como números; dos
 * alfanuméricos, en orden ASCII.
 */
function compareIdentifiers(a: string | number, b: string | number): number {
  const aNum = typeof a === 'number'
  const bNum = typeof b === 'number'
  if (aNum && bNum) return Math.sign((a as number) - (b as number))
  if (aNum) return -1
  if (bNum) return 1
  return a < b ? -1 : a > b ? 1 : 0
}

/**
 * Compara dos versiones según la **precedencia** de SemVer 2.0.0 (regla 11).
 *
 * @remarks
 * - Se comparan MAJOR, MINOR y PATCH numéricamente.
 * - Con igual núcleo, una pre-release tiene **menor** precedencia que la final
 *   (`1.0.0-rc.1 < 1.0.0`).
 * - Los metadatos de build se **ignoran** (`1.0.0+a` y `1.0.0+b` son iguales en precedencia).
 *
 * @param a - Primera versión (texto o {@link SemVer}).
 * @param b - Segunda versión (texto o {@link SemVer}).
 * @returns `-1` si `a < b`, `0` si tienen igual precedencia, `1` si `a > b`.
 * @throws `Error` si alguna versión en texto no es válida.
 *
 * @example
 * ```ts
 * ['1.0.0', '1.0.0-alpha', '1.0.0-beta.2'].sort(compare)
 * // ['1.0.0-alpha', '1.0.0-beta.2', '1.0.0']
 * ```
 */
export function compare(a: string | SemVer, b: string | SemVer): number {
  const va = typeof a === 'string' ? parse(a) : a
  const vb = typeof b === 'string' ? parse(b) : b

  for (const k of ['major', 'minor', 'patch'] as const) {
    if (va[k] !== vb[k]) return va[k] < vb[k] ? -1 : 1
  }

  const pa = va.prerelease
  const pb = vb.prerelease
  if (!pa.length && !pb.length) return 0
  if (!pa.length) return 1 // la versión final gana a cualquier pre-release
  if (!pb.length) return -1

  for (let i = 0; i < Math.max(pa.length, pb.length); i++) {
    // Un conjunto de identificadores más largo gana si los anteriores son iguales (11.4.4).
    if (i >= pa.length) return -1
    if (i >= pb.length) return 1
    const c = compareIdentifiers(pa[i], pb[i])
    if (c !== 0) return c
  }
  return 0
}

/**
 * Devuelve la siguiente versión al incrementar una parte.
 *
 * @remarks
 * Las partes inferiores vuelven a 0 (reglas 7 y 8) y se eliminan la
 * pre-release y el build. Durante el desarrollo inicial (`0.y.z`) el proyecto
 * sube `minor` para cambios incompatibles (PLAN.md §10.1).
 *
 * @param v - Versión de partida (texto o {@link SemVer}).
 * @param part - Parte a incrementar.
 * @returns La nueva versión en texto.
 *
 * @example
 * ```ts
 * bump('1.4.2', 'minor') // '1.5.0'
 * bump('1.4.2', 'major') // '2.0.0'
 * ```
 */
export function bump(v: string | SemVer, part: ReleaseType): string {
  const s = typeof v === 'string' ? parse(v) : v
  const next =
    part === 'major'
      ? { major: s.major + 1, minor: 0, patch: 0 }
      : part === 'minor'
        ? { major: s.major, minor: s.minor + 1, patch: 0 }
        : { major: s.major, minor: s.minor, patch: s.patch + 1 }
  return format({ ...next, prerelease: [], build: [] })
}
