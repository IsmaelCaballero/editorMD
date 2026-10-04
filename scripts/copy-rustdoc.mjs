// Copia la documentación generada por `cargo doc` a docs/api/rust para enlazarla desde la web del proyecto.
import { cpSync, existsSync, rmSync, writeFileSync } from 'node:fs'

const src = 'src-tauri/target/doc'
const dst = 'docs/api/rust'
if (!existsSync(src)) {
  console.error(`No existe ${src}: ejecuta antes "cargo doc".`)
  process.exit(1)
}
rmSync(dst, { recursive: true, force: true })
cpSync(src, dst, { recursive: true })
// Página de entrada que redirige a la documentación del crate.
writeFileSync(`${dst}/index.html`, '<meta http-equiv="refresh" content="0; url=editormd_lib/index.html">')
console.log(`Documentación Rust copiada a ${dst}`)
