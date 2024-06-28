module.exports = () => {
  console.log("override.config.js");

  return {
    name: "override_config",
    mode: "development",
    entry: "./src/index.js",
  };
};
