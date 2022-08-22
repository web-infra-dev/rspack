module.exports = {
  entry: {
    main: "./index.js",
  },
  builtins: {
    css: {
      presetEnv: ["chrome >= 40", "firefox > 10"],
    },
  },
};
