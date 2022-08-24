module.exports = {
  mode: "development",
  entry: {
    main: "./src/index.jsx",
  },
  output: {
    publicPath: "http://localhost:3000",
  },
  define: {
    "process.env.NODE_ENV": "development",
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
