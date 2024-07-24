module.exports = (env, argv) => {
  console.log({ argv });
  const { mode } = argv;
  return {
    entry: "./a.js",
    output: {
      filename: mode === "production" ? "prod.js" : "dev.js",
    },
  };
};
