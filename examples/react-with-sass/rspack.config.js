module.exports = {
  mode: "development",
  entry: {
    main: {
      import: ["./src/index.jsx"],
    }
  },
  output: {
    publicPath: "http://localhost:3000",
  },
  module: {
    rules: [
      {
        test: /\.s[ac]ss$/,
        use: [
          { loader: "builtin:sass-loader" }
        ],
        type: "css",
      },
    ],
  },
  builtins: {
    html: [{}],
  },
};
