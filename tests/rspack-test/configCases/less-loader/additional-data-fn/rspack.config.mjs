import { relative } from 'node:path';

/** @type {import("@rspack/core").Configuration} */
export default {
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
              additionalData: (content, loaderContext) => {
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
};
