module.exports = {
  mode: "development",
  entry: {
    main: {
      import: ["./src/index.tsx"],
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
  builtins: {
    html: [{}],
  },
};
