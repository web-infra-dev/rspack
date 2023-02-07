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
        use: [{ builtinLoader: "builtin:sass-loader" }],
        type: "css",
      },
    ],
  },
  builtins: {
    html: [{}],
  },
};
