import { defineConfig } from '@rspack/cli';
import { rspack } from '@rspack/core';

const config = (strictThisContextOnImports: boolean, i: number) =>
  defineConfig({
    output: {
      filename: `bundle${i}.js`,
    },
    module: {
      parser: {
        javascript: {
          strictThisContextOnImports,
        },
      },
    },
    optimization: {
      concatenateModules: false,
    },
    plugins: [
      new rspack.DefinePlugin({
        STRICT_THIS_CONTEXT_ON_IMPORTS: JSON.stringify(
          strictThisContextOnImports,
        ),
      }),
    ],
  });

export default [true, false].map((strictThisContextOnImports, i) =>
  config(strictThisContextOnImports, i),
);
