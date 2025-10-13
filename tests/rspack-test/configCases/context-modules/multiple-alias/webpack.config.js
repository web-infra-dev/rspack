"use strict";

const path = require("path");
const webpack = require("@rspack/core");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	resolve: {
		alias: {
			app: [path.join(__dirname, "src/main"), path.join(__dirname, "src/foo")]
		}
	},
	plugins: [
		new webpack.ContextReplacementPlugin(/main/, (context) => {
			Object.assign(context, {
				resource: ["../override"] // resolved relatively
			});
		})
	]
};
