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
  node: {
    __dirname: false,
    __filename: false,
  },
  optimization: {
    concatenateModules: true,
    usedExports: true,
    providedExports: true,
    minimize: false,
    mangleExports: false,
    inlineExports: false,
  },
});
