module.exports = () => {
  console.log("derived.webpack.config.js");

  return [
    {
      name: "derived_config1",
      entry: "./src/index1.js",
    },
    {
      name: "derived_config2",
      entry: "./src/index2.js",
    },
  ];
};
