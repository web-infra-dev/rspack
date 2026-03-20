import { defineConfig, ts } from '@rslint/core';

export default defineConfig([
  {
    // Global ignores — entry with only `ignores` excludes matching files from all rules
    ignores: [
      'packages/rspack/src/runtime/moduleFederationDefaultRuntime.js',
      'packages/rspack/compiled/**',
    ],
  },
  ts.configs.recommended,
  {
    languageOptions: {
      parserOptions: {
        project: ['./packages/rspack/tsconfig.json'],
      },
    },
    rules: {
      '@typescript-eslint/no-explicit-any': 'off',
      '@typescript-eslint/no-unused-vars': 'off',
      '@typescript-eslint/no-this-alias': 'off',
      '@typescript-eslint/ban-ts-comment': 'off',
      '@typescript-eslint/require-await': 'error',
      '@typescript-eslint/return-await': 'error',
      '@typescript-eslint/default-param-last': 'error',
      '@typescript-eslint/prefer-literal-enum-member': [
        'error',
        { allowBitwiseExpressions: true },
      ],
      'no-constant-binary-expression': 'off',
      'no-empty': 'off',
    },
  },
]);
