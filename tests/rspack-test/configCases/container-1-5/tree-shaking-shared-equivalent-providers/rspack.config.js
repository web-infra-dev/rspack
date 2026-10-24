const {
  DefinePlugin,
  container: { ModuleFederationPlugin },
} = require('@rspack/core');

module.exports = [
  'inferred-first',
  'explicit-first',
  'equivalent-filename',
  'different-scope-shape',
  'different-filenames',
].map((name) => {
  const common = {
    import: 'pkg-a',
    shareKey: 'same-key',
    requiredVersion: false,
    treeShaking: { mode: 'runtime-infer' },
  };
  const explicit = {
    ...common,
    version: '1.0.0',
    ...(name === 'equivalent-filename' && {
      treeShaking: { mode: 'runtime-infer', filename: '1.0.0/share-entry.js' },
    }),
    ...(name === 'different-scope-shape' && { shareScope: ['default'] }),
    ...(name === 'different-filenames' && {
      treeShaking: { mode: 'runtime-infer', filename: 'alternate-entry.js' },
    }),
  };
  return {
    target: 'async-node',
    output: { publicPath: 'PUBLIC_PATH' },
    plugins: [
      new DefinePlugin({
        CASE_NAME: JSON.stringify(name),
        EXPECTED_ARTIFACTS: name.startsWith('different-') ? 2 : 1,
      }),
      new ModuleFederationPlugin({
        name: `equivalent_${name}`,
        library: { type: 'commonjs-module' },
        manifest: { fileName: `${name}.json` },
        treeShakingSharedDir: `independent-${name}`,
        shared: (name === 'explicit-first'
          ? [explicit, common]
          : [common, explicit]
        ).map((options) => ({ 'alias-a': options })),
      }),
    ],
  };
});
