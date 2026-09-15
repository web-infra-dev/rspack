const entries = ['main', 'secondary', 'third'];

module.exports = ['function', 'string', 'entry'].map((kind) => ({
  target: 'node',
  entry: Object.fromEntries(
    entries.map((name) => [
      name,
      kind === 'entry'
        ? { import: './index.js', runtime: name === 'main' ? 'runtime' : false }
        : './index.js',
    ]),
  ),
  output: {
    filename: ({ chunk }) => {
      expect(chunk.name).toBeTruthy();
      return `${chunk.name}.js`;
    },
    chunkFilename: 'async-[name].js',
  },
  optimization: {
    minimize: false,
    chunkIds: 'named',
    runtimeChunk:
      kind === 'entry'
        ? false
        : {
            name:
              kind === 'string'
                ? ''
                : ({ name }) => (name === 'main' ? 'runtime' : ''),
          },
  },
  plugins: [
    (compiler) => {
      compiler.hooks.compilation.tap('CheckRuntime', (compilation) => {
        compilation.hooks.processAssets.tap('CheckRuntime', () => {
          for (const name of entries) {
            const entrypoint = compilation.entrypoints.get(name);
            const separate = kind !== 'string' && name === 'main';
            const runtimeName = separate ? 'runtime' : name;
            const runtimeChunk = entrypoint.getRuntimeChunk();
            expect(runtimeChunk.name).toBe(runtimeName);
            expect(entrypoint.chunks).toHaveLength(separate ? 2 : 1);
            expect(
              typeof runtimeChunk.runtime === 'string'
                ? [runtimeChunk.runtime]
                : [...runtimeChunk.runtime],
            ).toEqual([runtimeName]);
          }
          expect(Boolean(compilation.namedChunks.get(''))).toBe(false);
        });
      });
    },
  ],
}));
