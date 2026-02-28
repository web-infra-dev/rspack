"use strict";

const { rspack } = require("@rspack/core");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	optimization: {
		concatenateModules: true
	},
	plugins: [
		new rspack.DefinePlugin({
			PROPERTY: JSON.stringify("foo")
		})
	]
};
