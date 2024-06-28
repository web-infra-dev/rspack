module.exports = () => {
  console.log("other.config.js");

  return {
    name: "other_config",
    mode: "development",
    experiments: {
      topLevelAwait: true,
    },
  };
};
