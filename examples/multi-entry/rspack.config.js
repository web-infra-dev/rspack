module.exports = {
  mode: "development",
  entry: {
    index: {
      import: ["./index.js"],
    },
    second: {
      import: ["./second.js"],
    },
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
