const path = require("path");

/** @type {import("@rspack/core").LoaderDefinition} */
module.exports = function () {
	const callback = this.async();

	this.resolve(this.context, "./b.js", (err, result) => {
		callback(err, `module.exports = ${JSON.stringify(path.basename(/** @type {string} */(result)))};`)
	});
};
