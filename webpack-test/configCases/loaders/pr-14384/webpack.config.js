const PluginWithLoader = require("./PluginWithLoader.js");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	plugins: [new PluginWithLoader()]
};
