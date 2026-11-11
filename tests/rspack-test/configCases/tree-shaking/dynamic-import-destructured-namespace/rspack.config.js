const { DefinePlugin } = require('@rspack/core');
module.exports = [false, true].map((strict) => ({
  mode: 'production',
  optimization: { minimize: true },
  plugins: [
    new DefinePlugin({ STRICT_IMPORT_RECEIVER: JSON.stringify(strict) }),
  ],
  module: { parser: { javascript: { strictThisContextOnImports: strict } } },
  output: { chunkFilename: strict ? 'strict/[name].js' : 'loose/[name].js' },
}));
