import { defineConfig, definePlugin } from '@rspack/cli';
import type { Configuration } from '@rspack/core';

function config(
  index: number,
  { concatenateModules }: { concatenateModules?: boolean } = {},
): Configuration {
  return {
    entry: './index.js',
    output: {
      filename: `bundle.${index}.js`,
      pathinfo: false,
    },
    resolve: {
      extensions: ['.ts', '...'],
    },
    module: {
      rules: [
        {
          test: /\.ts$/,
          use: [
            {
              loader: 'builtin:swc-loader',
              options: {
                detectSyntax: 'auto',
                jsc: {
                  target: 'esnext',
                },
                collectTypeScriptInfo: {
                  exportedEnum: true,
                },
              },
            },
          ],
        },
      ],
    },
    plugins: [
      definePlugin(function (compiler) {
        new compiler.rspack.DefinePlugin({
          CONCATENATED: JSON.stringify(concatenateModules),
        }).apply(compiler);
      }),
    ],
    optimization: {
      concatenateModules,
      moduleIds: 'named',
      inlineExports: true,
    },
  };
}

export default defineConfig([
  config(0, { concatenateModules: true }),
  config(1, { concatenateModules: false }),
]);
