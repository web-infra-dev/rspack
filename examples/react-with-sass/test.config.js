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
        test : {
          type: "regexp",
          matcher: '\\.s[ac]ss$'
        },
        use: [{ builtinLoader: "sass-loader" }],
        type: "css",
      },
    ],
  },
  builtins: {
    html: [{}],
  },
};
