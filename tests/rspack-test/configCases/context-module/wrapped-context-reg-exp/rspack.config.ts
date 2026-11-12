import { defineConfig } from '@rspack/cli';

export default [
  defineConfig({
    module: {
      parser: {
        javascript: {
          wrappedContextRegExp: /.*1/,
        },
      },
    },
  }),
  defineConfig({
    module: {
      rules: [{ parser: { wrappedContextRegExp: /.*1/ } }],
    },
  }),
];
