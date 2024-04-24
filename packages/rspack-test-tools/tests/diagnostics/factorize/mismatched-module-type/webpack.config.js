const path = require("path");
const resolve = filename => path.resolve(__dirname, filename);

/**
 * @type {import('@rspack/core').RspackOptions}
 */
module.exports = {
	context: __dirname,
	module: {
		rules: [
      {
        test: resolve("app.jsx"),
        type: "js"
      },
      {
        test: resolve("app.tsx"),
        type: "ts"
      },
		]
	},
};
