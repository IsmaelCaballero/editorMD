// La versión de la app vive en tres ficheros (PLAN.md §10.1): esta prueba
// impide que se desincronicen y comprueba que cumplen SemVer 2.0.0 (con M17).
import { readFileSync } from 'node:fs'
import { join } from 'node:path'
import { describe, expect, it } from 'vitest'
import { isValid } from '../src/model/semver'

const root = join(__dirname, '..')
const read = (file: string) => readFileSync(join(root, file), 'utf-8')

const versions = {
  'package.json': (JSON.parse(read('package.json')) as { version: string }).version,
  'src-tauri/tauri.conf.json': (
    JSON.parse(read('src-tauri/tauri.conf.json')) as { version: string }
  ).version,
  'src-tauri/Cargo.toml': /^version\s*=\s*"([^"]+)"/m.exec(read('src-tauri/Cargo.toml'))?.[1],
}

describe('versión de la aplicación', () => {
  it.each(Object.entries(versions))('%s tiene una versión SemVer 2.0.0 válida (%s)', (_f, v) => {
    expect(isValid(v ?? '')).toBe(true)
  })

  it('las tres fuentes coinciden', () => {
    expect(new Set(Object.values(versions)).size).toBe(1)
  })

  it('el CHANGELOG tiene una sección para la versión actual', () => {
    expect(read('CHANGELOG.md')).toContain(`## [${versions['package.json']}]`)
  })
})
