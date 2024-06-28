module.exports = () => {
  return new Promise((resolve) => {
    setTimeout(() => {
      resolve({
        entry: "./a",
        output: {
          path: __dirname + "/binary",
          filename: "functor.js",
        },
      });
    });
  }, 0);
};
