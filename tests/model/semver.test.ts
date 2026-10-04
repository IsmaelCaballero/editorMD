// Tests unitarios de M17 SemVer: cubren las reglas 2, 9, 10 y 11 de semver.org.
import { describe, expect, it } from 'vitest'
import { bump, compare, format, isValid, parse } from '../../src/model/semver'

describe('parse', () => {
  it('descompone núcleo, pre-release y build', () => {
    expect(parse('1.0.0-rc.1+build.5')).toEqual({
      major: 1,
      minor: 0,
      patch: 0,
      prerelease: ['rc', 1],
      build: ['build', '5'],
    })
  })

  it('acepta una versión sin pre-release ni build', () => {
    expect(parse('0.0.1')).toEqual({ major: 0, minor: 0, patch: 1, prerelease: [], build: [] })
  })

  it('los identificadores alfanuméricos con dígitos siguen siendo texto', () => {
    expect(parse('1.0.0-x7.27a.0').prerelease).toEqual(['x7', '27a', 0])
  })

  it.each([
    '1.2', // faltan partes
    '1.2.3.4',
    '01.2.3', // cero a la izquierda (regla 2)
    '1.2.3-01', // cero a la izquierda en pre-release numérica (regla 9)
    '1.2.3-', // pre-release vacía
    '1.2.3+', // build vacío
    'v1.2.3', // el prefijo v no forma parte de SemVer
    '1.2.3-beta..1',
    '',
  ])('rechaza "%s"', (bad) => {
    expect(() => parse(bad)).toThrow(/no válida/)
    expect(isValid(bad)).toBe(false)
  })

  it('isValid acepta versiones correctas', () => {
    expect(isValid('10.20.30-alpha-1.beta+meta-data.001')).toBe(true)
  })
})

describe('format', () => {
  it.each(['1.2.3', '1.0.0-alpha.1', '1.0.0+20261003', '2.0.0-rc.2+sha.a1b2'])(
    'parse→format es idempotente para %s',
    (v) => {
      expect(format(parse(v))).toBe(v)
    },
  )
})

describe('compare (precedencia, regla 11)', () => {
  it('ordena el ejemplo oficial de la especificación', () => {
    const ordered = [
      '1.0.0-alpha',
      '1.0.0-alpha.1',
      '1.0.0-alpha.beta',
      '1.0.0-beta',
      '1.0.0-beta.2',
      '1.0.0-beta.11',
      '1.0.0-rc.1',
      '1.0.0',
    ]
    const shuffled = [...ordered].reverse()
    expect(shuffled.sort(compare)).toEqual(ordered)
  })

  it('compara major, minor y patch numéricamente (no como texto)', () => {
    expect(compare('1.10.0', '1.9.0')).toBe(1)
    expect(compare('2.0.0', '10.0.0')).toBe(-1)
    expect(compare('1.0.9', '1.0.10')).toBe(-1)
  })

  it('ignora los metadatos de build', () => {
    expect(compare('1.0.0+a', '1.0.0+b')).toBe(0)
  })

  it('la versión final gana a su pre-release y viceversa', () => {
    expect(compare('1.0.0', '1.0.0-rc.1')).toBe(1)
    expect(compare('1.0.0-rc.1', '1.0.0')).toBe(-1)
  })

  it('numérico < alfanumérico; alfanuméricos en orden ASCII', () => {
    expect(compare('1.0.0-1', '1.0.0-a')).toBe(-1)
    expect(compare('1.0.0-a', '1.0.0-1')).toBe(1)
    expect(compare('1.0.0-B', '1.0.0-a')).toBe(-1)
  })

  it('igualdad exacta y uso con objetos SemVer', () => {
    expect(compare(parse('3.1.4-x.1'), '3.1.4-x.1')).toBe(0)
  })
})

describe('bump', () => {
  it.each([
    ['1.4.2', 'patch', '1.4.3'],
    ['1.4.2', 'minor', '1.5.0'],
    ['1.4.2', 'major', '2.0.0'],
    ['0.0.1', 'minor', '0.1.0'],
    ['1.0.0-rc.1+b.7', 'patch', '1.0.1'],
  ] as const)('%s + %s = %s', (from, part, expected) => {
    expect(bump(from, part)).toBe(expected)
  })

  it('el resultado siempre tiene más precedencia que el origen', () => {
    for (const part of ['major', 'minor', 'patch'] as const) {
      expect(compare(bump('2.3.4', part), '2.3.4')).toBe(1)
    }
  })
})
