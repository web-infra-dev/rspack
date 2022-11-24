module.exports = {
  mode: "development",
  entry: {
    main: ["./src/index.jsx"],
  },
  output: {
    publicPath: "http://localhost:3000",
  },
  module: {
    rules: [
      {
        test : {
          type: "regexp",
          matcher: '\\.s[ac]ss$'
        },
        uses: [{ builtinLoader: "sass-loader" }],
        type: "css",
      },
    ],
  },
  builtins: {
    html: [{}],
  },
};
