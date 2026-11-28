import js from '@eslint/js'
import prettier from 'eslint-config-prettier'
import reactHooks from 'eslint-plugin-react-hooks'
import simpleImportSort from 'eslint-plugin-simple-import-sort'
import globals from 'globals'
import tseslint from 'typescript-eslint'

export default tseslint.config(
  {
    ignores: ['dist/**', 'coverage/**', 'node_modules/**', 'src/types/generated/**'],
  },
  js.configs.recommended,
  ...tseslint.configs.recommended,
  {
    files: ['**/*.{ts,tsx}'],
    languageOptions: {
      ecmaVersion: 2022,
      globals: { ...globals.browser, ...globals.es2022 },
    },
    plugins: { 'react-hooks': reactHooks, 'simple-import-sort': simpleImportSort },
    rules: {
      ...reactHooks.configs.recommended.rules,
      // These compiler-oriented rules require larger hook rewrites. The web
      // refactor will remove the patterns before they are enabled as errors.
      'react-hooks/set-state-in-effect': 'off',
      'react-hooks/refs': 'off',
      'react-hooks/incompatible-library': 'off',
      'simple-import-sort/imports': [
        'error',
        {
          groups: [
            ['^react$', '^react-dom', '^react-router'],
            ['^@?\\w'],
            ['^@/types(/.*)?$'],
            ['^@/lib(/.*)?$'],
            ['^@/store(/.*)?$'],
            ['^@/components/ui(/.*)?$'],
            ['^@/components(/.*)?$'],
            ['^@/features(/.*)?$'],
            ['^@/pages(/.*)?$'],
            ['^@/app(/.*)?$'],
            ['^\\.'],
            ['^\\u0000'],
          ],
        },
      ],
      'simple-import-sort/exports': 'error',
      '@typescript-eslint/consistent-type-imports': 'error',
      '@typescript-eslint/no-unused-vars': [
        'error',
        { argsIgnorePattern: '^_', varsIgnorePattern: '^_' },
      ],
    },
  },
  {
    files: ['**/*.test.{ts,tsx}', 'src/test/**/*.{ts,tsx}'],
    languageOptions: { globals: { ...globals.node } },
  },
  {
    files: ['public/**/*.js'],
    languageOptions: { globals: { ...globals.browser } },
  },
  {
    files: ['*.cjs'],
    languageOptions: { globals: { ...globals.node } },
  },
  prettier,
)
