/**
 * @type {import('@rspack/cli').Configuration}
 */
module.exports = {
  mode: "development",
  entry: {
    main: {
      import: ["./index.js"],
    }
  },
  output: {
    publicPath: "http://localhost:3000",
  },
  define: {
    "process.env.NODE_ENV": "'development'",
  },
  module: {
    rules: [],
    parser: {
      asset: {
        dataUrlCondition: {
          maxSize: 1,
        },
      },
    },
  },
  optimization: {
    splitChunks: {
      cacheGroups: {
        vendor: {
          chunks: "all",
          name: "vendor",
          test: "common"
        }
      }
    }
  },
  builtins: {
    html: [{}],
  },
};
