import type { LoaderContext } from '@rspack/core';
import { defineConfig } from '@rspack/cli';
import { relative } from 'node:path';

export default defineConfig({
  externals: {
    fs: 'node-commonjs fs',
    path: 'node-commonjs path',
  },
  target: 'web',
  node: false,
  module: {
    rules: [
      {
        test: /\.less$/,
        use: [
          {
            loader: 'less-loader',
            options: {
              additionalData: (
                content: string,
                loaderContext: LoaderContext,
              ) => {
                const { resourcePath, rootContext } = loaderContext;
                const relativePath = relative(rootContext, resourcePath);

                return `
										@background: coral;
										${content};
										.custom-class {
											color: red;
											relative-path: '${relativePath}';
										};
									`;
              },
            },
          },
        ],
        type: 'css',
        generator: {
          exportsOnly: false,
        },
      },
    ],
  },
});
