module.exports = {
  mode: "development",
  entry: {
    main: "./src/index.jsx",
  },
  module: {
    rules: [
      {
        test: "\\.s[ac]ss$",
        uses: [{ builtinLoader: "sass-loader" }],
        type: "css",
      },
    ],
  },
  builtins: {
    html: [{}],
  },
};
