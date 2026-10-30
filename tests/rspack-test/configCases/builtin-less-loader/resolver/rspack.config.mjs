import path from 'node:path';
export default ['builtin:less-loader', 'less-loader'].map((loader) => ({
  resolve: {
    alias: {
      '@theme': path.join(import.meta.dirname, 'theme'),
      collision: path.join(import.meta.dirname, 'wrong.less'),
    },
    modules: [path.join(import.meta.dirname, 'modules'), 'node_modules'],
    conditionNames: ['custom', '...'],
  },
  module: {
    rules: [
      {
        test: /\.less$/,
        type: 'asset/source',
        use: [
          './assert-dependencies-loader.mjs',
          {
            loader,
            options: {
              lessOptions: {
                paths: [path.join(import.meta.dirname, 'includes')],
              },
            },
          },
        ],
      },
    ],
  },
}));
