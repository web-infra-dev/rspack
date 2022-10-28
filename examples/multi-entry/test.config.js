module.exports = {
  mode: "development",
  entry: {
    index: "./index.js",
    second: "./second.js",
  },
  output: {
    publicPath: "http://localhost:3000",
  },
  define: {
    "process.env.NODE_ENV": "'development'",
  },
  builtins: {
    html: [{}],
  },
};
