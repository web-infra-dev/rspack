module.exports = () => [
  {
    output: {
      filename: "./dist-first.js",
    },
    name: "first",
    entry: "./src/first.js",
    mode: "development",
    stats: "minimal",
  },
  {
    output: {
      filename: "./dist-second.js",
    },
    name: "second",
    entry: "./src/second.js",
    mode: "development",
    stats: "minimal",
  },
];
