module.exports = [
  new Promise((resolve) => {
    setTimeout(() => {
      resolve({
        entry: "./a",
        name: "first",
        output: {
          path: __dirname + "/binary",
          filename: "a-promise.js",
        },
      });
    }, 0);
  }),
  new Promise((resolve) => {
    setTimeout(() => {
      resolve({
        entry: "./b",
        name: "second",
        output: {
          path: __dirname + "/binary",
          filename: "b-promise.js",
        },
      });
    }, 0);
  }),
];
