"use strict";

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	target: "web",
	mode: "development",
	devtool: false,
	module: {
		rules: [
			{
				test: /resource\.png$/,
				type: "asset/resource"
			},
			{
				test: /inline\.png$/,
				type: "asset/inline"
			},
			{
				test: /source\.text$/,
				type: "asset/source"
			},
			{
				mimetype: "text/html",
				type: "asset/resource"
			},
			{
				mimetype: "image/svg",
				type: "asset/resource"
			},
			{
				mimetype: "image/gif",
				type: "asset/resource"
			},
			{
				mimetype: "image/png",
				type: "asset/resource"
			},
			{
				test: /\.css$/,
				type: "css/auto"
			}
		]
	},
	output: {
		assetModuleFilename: "[name][ext]"
	},
	externals: {
		"shared-external.png": "asset shared-external.png"
	}
};
