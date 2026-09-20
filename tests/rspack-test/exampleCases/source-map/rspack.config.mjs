import path from 'node:path';

export default [
  'eval',
  'eval-cheap-source-map',
  'eval-cheap-module-source-map',
  'eval-source-map',
  'cheap-source-map',
  'cheap-module-source-map',
  'inline-cheap-source-map',
  'inline-cheap-module-source-map',
  'source-map',
  'inline-source-map',
  'hidden-source-map',
  'nosources-source-map',
].map((devtool) => ({
  mode: 'development',
  entry: {
    bundle: 'coffee-loader!./example.coffee',
  },
  output: {
    path: path.join(import.meta.dirname, 'dist'),
    filename: `./[name]-${devtool}.js`,
  },
  devtool,
  optimization: {
    runtimeChunk: true,
  },
}));
