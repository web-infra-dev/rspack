import { defineConfig } from '@rspack/cli';

export default defineConfig({
  output: {
    environment: {
      arrowFunction: true,
      bigIntLiteral: false,
      const: false,
      destructuring: false,
      forOf: false,
      dynamicImport: true,
      module: false,
    },
  },
});
