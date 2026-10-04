import { readFileSync } from 'node:fs'
import { svelte } from '@sveltejs/vite-plugin-svelte'
import { defineConfig } from 'vitest/config'

// La versión de la app sale de package.json (fuente única en el frontend, SemVer 2.0.0).
const pkg = JSON.parse(readFileSync(new URL('./package.json', import.meta.url), 'utf-8')) as {
  version: string
}

// https://vite.dev/config/  ·  https://v2.tauri.app/start/frontend/vite/
export default defineConfig({
  plugins: [svelte()],
  // En los tests de componentes, Svelte debe resolverse en su versión de navegador.
  resolve: process.env.VITEST ? { conditions: ['browser'] } : undefined,
  define: {
    __APP_VERSION__: JSON.stringify(pkg.version),
  },
  // Tauri espera un puerto fijo y necesita ver los errores de Rust en la consola.
  clearScreen: false,
  server: {
    port: 5173,
    strictPort: true,
    watch: { ignored: ['**/src-tauri/**'] },
  },
  test: {
    include: ['tests/**/*.test.ts'],
    environment: 'node',
    coverage: {
      provider: 'v8',
      include: ['src/model/**', 'src/controller/**', 'src/adapters/**'],
      reporter: ['text', 'html'],
      thresholds: { lines: 85 },
    },
  },
})
