// Tests unitarios de M01 DocumentState (escritos ANTES de la implementación: TDD).
import { describe, expect, it, vi } from 'vitest'
import { DocumentState, UNTITLED } from '../../src/model/document'

describe('DocumentState · creación', () => {
  it('un documento nuevo está vacío, sin ruta y sin cambios', () => {
    const doc = DocumentState.createNew()
    expect(doc.content).toBe('')
    expect(doc.path).toBeNull()
    expect(doc.isModified).toBe(false)
    expect(doc.isUntitled).toBe(true)
  })

  it('usa UTF-8 y LF por defecto (valores recomendados para documentos nuevos, §2.5)', () => {
    const doc = DocumentState.createNew()
    expect(doc.encoding).toBe('utf-8')
    expect(doc.lineEnding).toBe('lf')
  })

  it('admite otros valores por defecto (preferencias del usuario)', () => {
    const doc = DocumentState.createNew({ encoding: 'windows-1252', lineEnding: 'crlf' })
    expect(doc.encoding).toBe('windows-1252')
    expect(doc.lineEnding).toBe('crlf')
  })

  it('fromFile carga contenido, ruta y formato sin marcarlo como modificado', () => {
    const doc = DocumentState.fromFile('# Hola', {
      path: '/home/ana/notas.md',
      encoding: 'utf-8-bom',
      lineEnding: 'crlf',
    })
    expect(doc.content).toBe('# Hola')
    expect(doc.path).toBe('/home/ana/notas.md')
    expect(doc.encoding).toBe('utf-8-bom')
    expect(doc.lineEnding).toBe('crlf')
    expect(doc.isModified).toBe(false)
    expect(doc.isUntitled).toBe(false)
  })
})

describe('DocumentState · cambios sin guardar', () => {
  it('editar el contenido lo marca como modificado', () => {
    const doc = DocumentState.createNew()
    doc.setContent('texto')
    expect(doc.isModified).toBe(true)
  })

  it('volver exactamente al contenido guardado quita la marca (p. ej. tras deshacer)', () => {
    const doc = DocumentState.fromFile('original', { path: 'a.md' })
    doc.setContent('cambiado')
    doc.setContent('original')
    expect(doc.isModified).toBe(false)
  })

  it('markSaved fija el contenido actual como guardado', () => {
    const doc = DocumentState.createNew()
    doc.setContent('borrador')
    doc.markSaved()
    expect(doc.isModified).toBe(false)
    expect(doc.content).toBe('borrador')
  })

  it('markSaved con ruta: «Guardar como» asigna la ruta', () => {
    const doc = DocumentState.createNew()
    doc.setContent('x')
    doc.markSaved('C:\\Docs\\informe.md')
    expect(doc.path).toBe('C:\\Docs\\informe.md')
    expect(doc.isUntitled).toBe(false)
  })

  it('cambiar la codificación o el fin de línea también es un cambio sin guardar', () => {
    const doc = DocumentState.fromFile('x', { path: 'a.md' })
    doc.setEncoding('iso-8859-15')
    expect(doc.isModified).toBe(true)
    doc.setEncoding('utf-8')
    expect(doc.isModified).toBe(false)
    doc.setLineEnding('crlf')
    expect(doc.isModified).toBe(true)
  })
})

describe('DocumentState · nombres y título de ventana', () => {
  it.each([
    ['/home/ana/notas.md', 'notas.md'],
    ['C:\\Users\\Ana\\Docs\\informe.Rmd', 'informe.Rmd'],
    ['relativo.md', 'relativo.md'],
    ['/carpeta/con espacios/mi doc.md', 'mi doc.md'],
  ])('fileName de %s es %s (rutas Unix y Windows)', (path, name) => {
    expect(DocumentState.fromFile('', { path }).fileName).toBe(name)
  })

  it('un documento sin ruta se llama «Sin título»', () => {
    expect(DocumentState.createNew().fileName).toBe(UNTITLED)
    expect(UNTITLED).toBe('Sin título')
  })

  it('el título de ventana añade * si hay cambios', () => {
    const doc = DocumentState.fromFile('a', { path: '/x/notas.md' })
    expect(doc.windowTitle('editorMD')).toBe('notas.md — editorMD')
    doc.setContent('b')
    expect(doc.windowTitle('editorMD')).toBe('notas.md * — editorMD')
  })
})

describe('DocumentState · notificación de cambios (Observer)', () => {
  it('avisa a los suscriptores cuando cambia el contenido', () => {
    const doc = DocumentState.createNew()
    const listener = vi.fn()
    doc.subscribe(listener)
    doc.setContent('hola')
    expect(listener).toHaveBeenCalledTimes(1)
    expect(listener).toHaveBeenCalledWith(doc)
  })

  it('no avisa si el contenido no cambia realmente', () => {
    const doc = DocumentState.fromFile('igual', { path: 'a.md' })
    const listener = vi.fn()
    doc.subscribe(listener)
    doc.setContent('igual')
    doc.setEncoding('utf-8')
    doc.setLineEnding('lf')
    expect(listener).not.toHaveBeenCalled()
  })

  it('avisa al guardar, porque cambia el estado de modificado', () => {
    const doc = DocumentState.createNew()
    doc.setContent('x')
    const listener = vi.fn()
    doc.subscribe(listener)
    doc.markSaved('a.md')
    expect(listener).toHaveBeenCalledTimes(1)
  })

  it('la función devuelta por subscribe cancela la suscripción', () => {
    const doc = DocumentState.createNew()
    const listener = vi.fn()
    const unsubscribe = doc.subscribe(listener)
    unsubscribe()
    doc.setContent('x')
    expect(listener).not.toHaveBeenCalled()
  })

  it('admite varios suscriptores independientes', () => {
    const doc = DocumentState.createNew()
    const a = vi.fn()
    const b = vi.fn()
    doc.subscribe(a)
    const offB = doc.subscribe(b)
    offB()
    doc.setContent('x')
    expect(a).toHaveBeenCalledTimes(1)
    expect(b).not.toHaveBeenCalled()
  })
})

describe('DocumentState · reutilización', () => {
  it('replaceWith carga otro documento en el mismo objeto y avisa', () => {
    const doc = DocumentState.createNew()
    doc.setContent('borrador')
    const listener = vi.fn()
    doc.subscribe(listener)
    doc.replaceWith(DocumentState.fromFile('otro', { path: 'b.md', lineEnding: 'crlf' }))
    expect(doc.content).toBe('otro')
    expect(doc.path).toBe('b.md')
    expect(doc.lineEnding).toBe('crlf')
    expect(doc.isModified).toBe(false)
    expect(listener).toHaveBeenCalledTimes(1)
  })

  it('reset vuelve a un documento nuevo vacío', () => {
    const doc = DocumentState.fromFile('algo', { path: 'a.md', encoding: 'ascii' })
    doc.setContent('más')
    doc.reset()
    expect(doc.content).toBe('')
    expect(doc.path).toBeNull()
    expect(doc.encoding).toBe('utf-8')
    expect(doc.isModified).toBe(false)
  })
})

describe('DocumentState · wordCount (v0.2.0)', () => {
  it.each([
    ['', 0],
    ['Hola', 1],
    ['# Hola mundo', 2],
    ['**negrita** y *cursiva*', 3],
    ['Ñandú, cigüeña; café.', 3],
    ['- [x] tarea 1\n- [ ] tarea 2', 4],
    ['日本語 text', 2],
  ])('%j tiene %i palabras', (content, words) => {
    expect(DocumentState.fromFile(content, { path: 'a.md' }).wordCount).toBe(words)
  })
})
