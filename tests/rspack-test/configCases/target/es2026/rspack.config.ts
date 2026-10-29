import { defineConfig } from '@rspack/cli';
import type { Compiler } from '@rspack/core';

export default defineConfig({
  target: ['web', 'es2026'],
  module: {
    rules: [
      {
        test: /\.js$/,
        use: 'builtin:swc-loader',
      },
    ],
  },
  optimization: {
    minimize: true,
  },
  plugins: [
    (compiler: Compiler) => {
      compiler.hooks.afterEnvironment.tap('CheckTarget', () => {
        expect(compiler.options.output.environment).toMatchObject({
          arrowFunction: true,
          const: true,
          optionalChaining: true,
          dynamicImport: true,
        });
        // Inherited SWC targets remain capped at the supported version.
        expect(compiler.target.esVersion).toBe(2024);
      });
    },
  ],
});
