const { rspack } = require('@rspack/core');

module.exports = {
  target: 'node',
  plugins: [
    new rspack.DefinePlugin({
      REPLACEMENT: `(() => {
				if (true) { function require() { return "replacement"; } }
				return require("./missing-replacement");
			})()`,
      REPLACEMENT_DEFAULT: `((value = "replacement default") => {
				if (true) { function require() { return value; } }
				return require("./missing-replacement-default");
			})()`,
    }),
  ],
};
