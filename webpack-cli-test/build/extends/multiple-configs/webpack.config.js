module.exports = () => {
  console.log("derived.webpack.config.js");

  return [
    {
      name: "derived_config1",
      extends: "./base.webpack.config.js",
      entry: "./src/index1.js",
    },
    {
      name: "derived_config2",
      extends: "./base.webpack.config.js",
      entry: "./src/index2.js",
    },
  ];
};
