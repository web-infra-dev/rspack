module.exports = {
  mode: "development",
  entry: {
    main: ["./src/index.jsx"],
  },
  define: {
    "process.env.NODE_ENV": "'development'",
  },
  builtins: {
    html: [{}],
  },
  module: {
    rules: [
      {
        test: /\.less$/,
        type: "asset"
      }
    ]
  }
};
