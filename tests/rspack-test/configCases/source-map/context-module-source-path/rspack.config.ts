import path from 'node:path';
import { defineConfig } from '@rspack/cli';
import { DefinePlugin } from '@rspack/core';

const contextSuffix = String.raw`|sync|/^\.\/.*\.js$/`;

export default defineConfig(
  [
    undefined,
    'identifier:///[absolute-resource-path]',
    'identifier:///[resource-path]',
  ].map((template) => ({
    node: {
      __dirname: false,
      __filename: false,
    },
    devtool: 'source-map' as const,
    output: { devtoolModuleFilenameTemplate: template },
    plugins: [
      new DefinePlugin({
        EXPECTED_CONTEXT_SOURCE:
          template === undefined
            ? 'undefined'
            : JSON.stringify(
                'identifier:///' +
                  (template.endsWith('[absolute-resource-path]')
                    ? path.join(import.meta.dirname, 'foo')
                    : './foo') +
                  contextSuffix,
              ),
      }),
    ],
  })),
);
