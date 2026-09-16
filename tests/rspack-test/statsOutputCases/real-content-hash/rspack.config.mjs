import path from 'node:path';

/** @type {import("@rspack/core").Configuration} */
const base = {
  mode: 'production',
  entry: {
    index: {
      import: './index',
      runtime: 'runtime',
    },
    a: './a',
    b: './b',
  },
  module: {
    generator: {
      asset: {
        filename: '[hash][ext][query]',
      },
    },
    rules: [
      {
        test: /\.(png|jpg)$/,
        type: 'asset/resource',
      },
    ],
  },
  optimization: {
    minimize: true,
  },
  stats: {
    relatedAssets: true,
    cachedAssets: true,
  },
};

/** @type {import("@rspack/core").Configuration[]} */
export default [
  {
    ...base,
    name: 'a-normal',
    context: path.resolve(import.meta.dirname, 'a'),
    output: {
      path: path.resolve(
        import.meta.dirname,
        '../../js/stats/real-content-hash/a-normal',
      ),
      filename: '[contenthash]-[contenthash:6].js',
    },
  },
  {
    ...base,
    name: 'b-normal',
    context: path.resolve(import.meta.dirname, 'b'),
    output: {
      path: path.resolve(
        import.meta.dirname,
        '../../js/stats/real-content-hash/b-normal',
      ),
      filename: '[contenthash]-[contenthash:6].js',
    },
  },
  {
    ...base,
    context: path.resolve(import.meta.dirname, 'a'),
    name: 'a-source-map',
    devtool: 'source-map',
    output: {
      path: path.resolve(
        import.meta.dirname,
        '../../js/stats/real-content-hash/a-source-map',
      ),
      filename: '[contenthash]-[contenthash:6].js',
    },
  },
  {
    ...base,
    context: path.resolve(import.meta.dirname, 'b'),
    name: 'b-source-map',
    devtool: 'source-map',
    output: {
      path: path.resolve(
        import.meta.dirname,
        '../../js/stats/real-content-hash/b-source-map',
      ),
      filename: '[contenthash]-[contenthash:6].js',
    },
  },
];
