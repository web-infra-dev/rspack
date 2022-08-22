module.exports = {
  mode: "development",
  entry: {
    main: "./index.js",
  },
  output: {
    publicPath: "http://localhost:3000",
  },
  define: {
    "process.env.NODE_ENV": "development",
  },
  builtins: {
    html: [{}],
  },
};
