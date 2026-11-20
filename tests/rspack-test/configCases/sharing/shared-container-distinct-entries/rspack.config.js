module.exports = {
  target: 'async-node',
  output: { enabledLibraryTypes: ['commonjs2'] },
  plugins: [
    {
      apply(compiler) {
        // Internal API on purpose: the regression needs two shared containers
        // with the same key, scope and version in one compiler. SharedContainerPlugin
        // is not exported, and IndependentSharedPlugin builds each fallback in
        // its own compiler, so the public options cannot reach this case.
        for (const name of ['A', 'B']) {
          compiler.__internal__registerBuiltinPlugin({
            name: 'SharedContainerPlugin',
            options: {
              name,
              request: `./${name.toLowerCase()}.js`,
              shareKey: 'pkg',
              shareScope: 'default',
              version: '1.0.0',
              fileName: `${name}.js`,
              library: { type: 'commonjs2' },
            },
            canInherentFromParent: false,
          });
        }
      },
    },
  ],
};
