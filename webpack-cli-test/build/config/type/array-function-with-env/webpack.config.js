module.exports = [
  (env, argv) => {
    const { mode } = argv;
    return {
      entry: "./a.js",
      name: "first",
      output: {
        filename: mode === "production" ? "a-prod.js" : "a-dev.js",
      },
    };
  },
  (env, argv) => {
    const { mode } = argv;
    return {
      entry: "./b.js",
      name: "second",
      output: {
        filename: mode === "production" ? "b-prod.js" : "b-dev.js",
      },
    };
  },
];
