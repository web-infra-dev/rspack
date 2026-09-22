/** @type {import("@rspack/core").LoaderDefinitionFunction} */
export default function (source) {
	const ref = JSON.parse(source);
	const callback = this.async();
	this.importModule("../loader.mjs!" + ref, {}, (err, exports) => {
		if (err) {
			callback(null, JSON.stringify(`err: ${err && err.message}`));
		} else {
			callback(null, JSON.stringify(`source: ${exports}`));
		}
	});
};
