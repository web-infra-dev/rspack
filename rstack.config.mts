import { define } from 'rstack';
import skillsLock from './skills-lock.json' with { type: 'json' };

define.fmt({
  singleQuote: true,
  ignorePatterns: [
    // Tests
    'tests/rspack-test/**/*',
    '!tests/rspack-test/**/',
    '!tests/rspack-test/**/rspack.config.*',
    '!tests/rspack-test/**/*.toml',
    'packages/**/etc/**/*',
    'tests/e2e/cases/make/rewrite-factorize-request/file.js',

    // Benchmark fixtures
    'xtask/benchmark/benches/fixtures/css/**',
    'xtask/benchmark/benches/fixtures/rspack_sources/**',

    // Crates
    'crates/**',
    '!crates/**/',
    '!crates/**/*.md',
    '!crates/**/*.toml',

    // Ignore installed Skills because their formatting may differ from this repository.
    ...Object.keys(skillsLock.skills).map((name) => `.agents/skills/${name}`),
  ],
  plugins: ['heading-case'],
  overrides: [
    {
      files: '*.toml',
      options: {
        plugins: ['prettier-plugin-toml'],
        printWidth: 120,
        keyValueEqualsSignAlignment: true,
        groupBlankLinesLimit: 2,
        stringQuoteStyle: 'preserve',
        keyQuoteStyle: 'preserve',
        commentStyle: 'preserve',
        trailingCommentAlignment: true,
        trailingCommentSpaceWidth: 1,
      },
    },
    {
      files: 'Cargo.toml',
      options: {
        // Tombi always expands arrays that exceed the line width.
        // Keep short dependency feature lists on one line.
        printWidth: 160,
      },
    },
    {
      files: '.cargo/config.toml',
      options: {
        printWidth: 160,
        trailingCommentAlignment: false,
      },
    },
    {
      files: 'deny.toml',
      options: {
        // Avoid aligning unrelated settings across explanatory comments.
        keyValueEqualsSignAlignment: false,
      },
    },
  ],
});

define.lint(({ js, ts, globals }) => [
  js.configs.recommended,
  ts.configs.recommended,
  {
    // Global ignores — entry with only `ignores` excludes matching files from all rules
    ignores: [
      'packages/rspack/src/runtime/moduleFederationDefaultRuntime.js',
      'xtask/benchmark/benches/fixtures/rspack_sources/**',
      '**/tests/**',
      // Imported resolver fixtures/examples contain intentionally odd JS
      'crates/rspack_resolver/**',
    ],
  },
  {
    languageOptions: {
      parserOptions: {
        project: ['./packages/*/tsconfig.json'],
      },
    },
    rules: {
      '@typescript-eslint/no-explicit-any': 'off',
      '@typescript-eslint/no-unused-vars': 'off',
      '@typescript-eslint/no-this-alias': 'off',
      '@typescript-eslint/ban-ts-comment': 'off',
      '@typescript-eslint/no-empty-object-type': 'off',
      '@typescript-eslint/no-unsafe-function-type': 'off',
      '@typescript-eslint/no-wrapper-object-types': 'off',
      '@typescript-eslint/require-await': 'error',
      '@typescript-eslint/return-await': 'error',
      '@typescript-eslint/default-param-last': 'error',
      '@typescript-eslint/prefer-literal-enum-member': [
        'error',
        { allowBitwiseExpressions: true },
      ],
      '@typescript-eslint/no-require-imports': 'off',
      '@typescript-eslint/triple-slash-reference': 'off',
      'no-constant-binary-expression': 'off',
      'no-control-regex': 'off',
      'no-empty': 'off',
      'no-prototype-builtins': 'off',
      'no-useless-assignment': 'off',
      'prefer-spread': 'off',
      'preserve-caught-error': 'off',
    },
  },
  // Enable no-undef for JS files
  {
    files: ['**/*.{js,jsx,mjs,cjs}'],
    rules: {
      'no-undef': 'error',
    },
    languageOptions: {
      globals: {
        ...globals.browser,
        ...globals.jest,
        ...globals.node,
        ...globals.rspack,
        ...globals.rstest,
        $: 'readonly',
        $IMPORT_META_NAME: 'readonly',
        $PATH: 'readonly',
        __prefresh_errors__: 'readonly',
        __prefresh_utils__: 'readonly',
        fs: 'readonly',
        path: 'readonly',
      },
    },
  },
  {
    files: ['**/*.d.ts'],
    rules: {
      'no-var': 'off',
    },
  },
]);

define.staged({
  '*.rs': 'rustfmt',
  '*.{md,mdx,json,css,less,scss,toml,yaml,yml}': 'rs fmt',
  '*.{ts,tsx,js,cts,cjs,mts,mjs}': ['rs lint', 'rs fmt'],
  'website/**/*': () => 'pnpm --dir website run check:spell',
});
