export default [
  { type: 'asset', parser: { dataUrlCondition: { maxSize: 1024 } } },
  { type: 'asset', parser: { dataUrlCondition: { maxSize: 0 } } },
  { type: 'asset/resource' },
].map((rule) => ({
  target: 'node',
  output: { publicPath: '', assetModuleFilename: '[name][ext]' },
  module: { rules: [{ test: /\.txt$/, ...rule }] },
}));
