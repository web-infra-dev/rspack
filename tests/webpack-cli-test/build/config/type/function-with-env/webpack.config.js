const { DefinePlugin } = require("webpack");

module.exports = (env) => {
  console.log(env);
  if (env.isProd) {
    return {
      entry: "./a.js",
      output: {
        filename: "prod.js",
      },
    };
  }
  if (env.foo === `''`) {
    return {
      entry: "./a.js",
      output: {
        filename: "empty-string.js",
      },
    };
  }
  if (env.foo === `bar=''`) {
    return {
      entry: "./a.js",
      output: {
        filename: "new-empty-string.js",
      },
    };
  }
  if (env.foo === "undefined") {
    return {
      entry: "./a.js",
      output: {
        filename: "undefined-foo.js",
      },
    };
  }
  return {
    entry: "./a.js",
    mode: "development",
    stats: env.verboseStats ? "verbose" : "normal",
    plugins: [
      new DefinePlugin({
        envMessage: env.envMessage ? JSON.stringify("env message present") : false,
      }),
    ],
    output: {
      filename: "dev.js",
    },
  };
};
