module.exports = () => {
  console.log("base.webpack.config.js");

  return {
    name: "base_config",
    mode: "development",
    entry: "./src/index.js",
  };
};
