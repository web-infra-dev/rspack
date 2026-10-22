import path from "node:path";

/** @type {import("@rspack/core").LoaderDefinition} */
export default function () {
	const callback = this.async();

	this.resolve(this.context, "./b.js", (err, result) => {
		callback(err, `module.exports = ${JSON.stringify(path.basename(/** @type {string} */(result)))};`)
	});
};
