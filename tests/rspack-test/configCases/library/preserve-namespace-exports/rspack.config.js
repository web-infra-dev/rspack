module.exports = [
  ['modern-module', 'modern-module'],
  ['shared-modern-module', 'modern-module'],
  ['module', 'module'],
].map(([name, type]) => ({
  name,
  entry: name.startsWith('shared-')
    ? { main: './index.js', other: './other.js' }
    : './index.js',
  mode: 'production',
  target: 'node',
  devtool: false,
  output: {
    module: true,
    filename: `${name}/[name].mjs`,
    chunkFilename: `${name}/[name].mjs`,
    library: { type },
  },
  optimization: {
    minimize: false,
    chunkIds: 'named',
    runtimeChunk: name.startsWith('shared-') ? 'single' : false,
    splitChunks: false,
  },
}));
