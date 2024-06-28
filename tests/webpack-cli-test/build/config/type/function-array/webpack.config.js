module.exports = () => [
  {
    entry: "./a",
    name: "first",
    output: {
      path: __dirname + "/binary",
      filename: "a-functor.js",
    },
  },
  {
    entry: "./b",
    name: "second",
    output: {
      path: __dirname + "/binary",
      filename: "b-functor.js",
    },
  },
];
