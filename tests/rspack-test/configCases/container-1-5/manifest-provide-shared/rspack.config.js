const { DefinePlugin, container, sharing } = require('@rspack/core');

module.exports = [undefined, './static.js', './lazy.js'].map(
  (request, index) => ({
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
        PROVIDED_REQUEST: JSON.stringify(request || ''),
      }),
      new container.ModuleFederationPlugin({
        name: `container${index}`,
        filename: `${index}-container.js`,
        library: { type: 'commonjs-module' },
        manifest: { fileName: `mf-${index}.json` },
        exposes: {
          './Mount': { import: './mount.js', name: 'expose-mount' },
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
  }),
);
