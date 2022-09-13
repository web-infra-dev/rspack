module.exports = {
  mode: "development",
  entry: {
    main: "./src/index.jsx",
  },
  output: {
    publicPath: "http://localhost:3000",
  },
  module: {
  },
  builtins: {
    html: [{}],
  },
};
