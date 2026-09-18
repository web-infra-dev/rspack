import path from "node:path";

/** @type {import("@rspack/core").LoaderDefinition} */
export default function () {
	const callback = this.async();
	this.importModule(
		path.resolve(import.meta.dirname, "module.js"),
		{ baseUri: "webpack://" },
		(error, exports) => {
			if (error) {
				callback(error);
				return;
			}

			callback(
				null,
				`module.exports = ${exports.asset ? JSON.stringify(exports.asset) : undefined
				}`
			);
		}
	);
};
