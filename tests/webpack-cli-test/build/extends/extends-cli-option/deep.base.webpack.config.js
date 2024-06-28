module.exports = () => {
  console.log("deep.base.webpack.config.js");

  return {
    name: "base_config",
    mode: "development",
    entry: "./src/index.js",
    bail: true,
  };
};
