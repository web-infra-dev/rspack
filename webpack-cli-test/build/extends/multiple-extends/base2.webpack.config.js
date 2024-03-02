module.exports = () => {
  console.log("base2.webpack.config.js");

  return {
    name: "base_config2",
    entry: "./src/index2.js",
    externals: {
      jquery: "jQuery",
    },
  };
};
