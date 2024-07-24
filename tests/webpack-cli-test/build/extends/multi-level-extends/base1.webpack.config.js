module.exports = () => {
  console.log("base1.webpack.config.js");

  return {
    name: "base_config1",
    extends: ["./base2.webpack.config.js"],
    mode: "production",
    entry: "./src/index1.js",
    output: {
      filename: "bundle1.js",
    },
  };
};
