import { defineConfig } from '@rspack/cli';
import { experiments } from '@rspack/core';

const expression = Array.from(
  { length: 1000 },
  (_, index) => `'a${index}' + f(${index})`,
).join(' +\n');

const build = `export const build = function (f) {
  return (${expression});
};`;

export default defineConfig({
  mode: 'development',
  module: {
    rules: [
      {
        test: /\.(js|jsx)$/,
        exclude: /node_modules/,
        use: {
          loader: 'builtin:swc-loader',
          options: {
            detectSyntax: 'auto',
            jsc: {
              target: 'esnext',
              transform: {
                reactCompiler: true,
                react: { runtime: 'automatic' },
              },
            },
          },
        },
      },
    ],
  },
  plugins: [
    new experiments.VirtualModulesPlugin({
      'plain.js': build,
      'opt-out.js': `export const build = function (f) {
        'use no memo';
        return (${expression});
      };`,
      // JSX forces the compiler to convert the file instead of skipping it.
      'with-react.jsx': `${build}
        export function Component({ value }) {
          return <div>{value}</div>;
        }`,
    }),
  ],
});
