"use strict";

const { rspack } = require("@rspack/core");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	mode: "development",
	plugins: [
    new rspack.container.ModuleFederationPluginV1({
      shared: ["./shared-esm-pkg/index.js"]
    }),
	]
};
