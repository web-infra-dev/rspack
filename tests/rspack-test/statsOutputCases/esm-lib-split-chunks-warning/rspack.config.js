const rspack = require("@rspack/core");
const LogTestPlugin = require("@rspack/test-tools/helper/legacy/LogTestPlugin");

module.exports = {
  entry: './index.js',
  plugins: [new LogTestPlugin(true)],
  stats: {
    logging: true,
  },
  output: {
    library: {
      type: "modern-module"
    }
  },
  optimization: {
    splitChunks: {
      maxSize: 100000, // unsupported field
      minSizeReduction: 50000, // unsupported field
      cacheGroups: {
        vendors: {
          test: /[\\/]node_modules[\\/]/,
          name: 'vendors',
          chunks: 'all',
          maxAsyncSize: 200000, // unsupported field
          maxAsyncRequests: 10, // unsupported field
          maxInitialRequests: 5, // unsupported field
        }
      }
    }
  }
};
