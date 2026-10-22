import { defineConfig } from '@rspack/cli';

export default defineConfig([
  {
    output: {
      globalObject: "null || new Function('return this')()",
    },
  },
  {
    output: {
      globalObject: "(new Function('return this'))()",
    },
  },
  {
    output: {
      globalObject: "1 > 2 ? null : new Function('return this')()",
    },
  },
]);
