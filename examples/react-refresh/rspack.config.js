module.exports = {
  mode: "development",
  entry: "./src/index.tsx",
  output: {
    publicPath: "http://localhost:3000",
  },
  module: {
    parser: {
      asset: {
        dataUrlCondition: {
          maxSize: 1,
        },
      },
    },
  },
  builtins: {
    html: [{}],
    define: {
      "process.env.NODE_ENV": "'development'",
    },
  },
};
