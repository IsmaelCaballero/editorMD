// Configuración de ESLint (formato "flat config", ESLint ≥ 9). F0.7 · PLAN.md §10.
// Reglas: recomendadas de JS + TypeScript + Svelte. Prettier se encarga del formato,
// por eso eslint-config-prettier desactiva las reglas de estilo que entrarían en conflicto.
import js from '@eslint/js'
import prettier from 'eslint-config-prettier'
import svelte from 'eslint-plugin-svelte'
import globals from 'globals'
import ts from 'typescript-eslint'

export default ts.config(
  {
    ignores: [
      'dist/',
      'coverage/',
      'docs/',
      'src-tauri/target/',
      'src-tauri/gen/',
      'node_modules/',
    ],
  },
  js.configs.recommended,
  ...ts.configs.recommended,
  ...svelte.configs.recommended,
  prettier,
  ...svelte.configs.prettier,
  {
    languageOptions: {
      globals: { ...globals.browser, ...globals.node },
    },
  },
  {
    files: ['**/*.svelte', '**/*.svelte.ts'],
    languageOptions: {
      parserOptions: { parser: ts.parser, extraFileExtensions: ['.svelte'] },
    },
  },
  {
    // TypeScript ya comprueba los identificadores no definidos (y conoce los .d.ts);
    // no-undef daría falsos positivos (recomendación oficial de typescript-eslint).
    files: ['**/*.ts', '**/*.svelte'],
    rules: { 'no-undef': 'off' },
  },
  {
    rules: {
      // Variables sin usar: error, salvo si empiezan por "_" (parámetros obligatorios no usados).
      '@typescript-eslint/no-unused-vars': [
        'error',
        { argsIgnorePattern: '^_', varsIgnorePattern: '^_' },
      ],
      // Preferir const y prohibir var.
      'prefer-const': 'error',
      'no-var': 'error',
      // == solo para comparar con null.
      eqeqeq: ['error', 'always', { null: 'ignore' }],
    },
  },
)
