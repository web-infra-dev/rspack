"use strict";

const rspack = require("@rspack/core");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	output: {
		assetModuleFilename: "assets/[name][ext]"
	},
  module: {
    rules: [
      {
        test: /\.woff2$/,
        type: 'asset/resource',
      }
    ]
  },
	plugins: [
		new rspack.container.ModuleFederationPluginV1({
			shared: [
				{
					import: "pkg/",
					requiredVersion: false,
				},
			],
		})
	]
};
