module.exports = async () => {
  return {
    entry: "./a",
    output: {
      path: __dirname + "/binary",
      filename: "functor.js",
    },
  };
};
