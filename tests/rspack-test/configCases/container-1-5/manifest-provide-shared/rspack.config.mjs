import { DefinePlugin, container, sharing } from '@rspack/core';
export default [
  undefined,
  './static.js',
  './lazy.js',
  undefined,
  './static.js',
  './lazy.js',
].map((request, index) => ({
  experiments: { css: true },
  module: {
    rules: [{ test: /\.css$/, type: 'css', generator: { exportsOnly: false } }],
  },
  externals: {
    fs: 'node-commonjs fs',
    path: 'node-commonjs path',
  },
  optimization: {
    chunkIds: 'named',
    moduleIds: 'named',
  },
  output: {
    chunkFilename: `${index}-[name].js`,
    cssChunkFilename: `${index}-[name].css`,
  },
  plugins: [
    new DefinePlugin({
      CASE_INDEX: index,
      NAMED_EXPOSE: index < 3,
      PROVIDED_REQUEST: JSON.stringify(request || ''),
    }),
    new container.ModuleFederationPlugin({
      name: `container${index}`,
      filename: `${index}-container.js`,
      library: { type: 'commonjs-module' },
      manifest: { fileName: `mf-${index}.json` },
      exposes: {
        './Mount':
          index < 3
            ? { import: './mount.js', name: 'expose-mount' }
            : './mount.js',
      },
    }),
    ...(request
      ? [
          new sharing.ProvideSharedPlugin({
            provides: { [request]: { shareKey: request, version: '1.0.0' } },
          }),
        ]
      : []),
  ],
}));
