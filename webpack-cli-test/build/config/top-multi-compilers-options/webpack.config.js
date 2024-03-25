class SimpleProgressWebpackPlugin {
  constructor(options) {
    this.options = options;
  }

  apply(compiler) {
    compiler.hooks.done.tap("test", () => {
      console.log("Done", this.options.name);
    });
  }
}

const configs = [];

for (let i = 0; i < 3; i++) {
  configs.push({
    mode: "development",
    name: `build${i}`,
    entry: "./index.js",
    plugins: [
      new SimpleProgressWebpackPlugin({
        name: `build${i}`,
      }),
    ],
  });
}

configs.push(async () => {
  return {
    mode: "development",
    name: `build${3}`,
    entry: "./index.js",
    plugins: [
      new SimpleProgressWebpackPlugin({
        name: `build${3}`,
        format: "simple",
      }),
    ],
  };
});

module.exports = configs;
module.exports.parallelism = 1;
