module.exports = {
  target: 'async-node',
  output: { enabledLibraryTypes: ['commonjs2'] },
  plugins: [
    {
      apply(compiler) {
        // Exercise two lower-level shared containers in one compiler. The public
        // IndependentSharedPlugin normally builds each in a separate child.
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
