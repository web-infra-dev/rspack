/** @type {import("@rspack/core").LoaderDefinitionFunction} */
export default function (source) {
	const ref = JSON.parse(source);
	const callback = this.async();
	this.loadModule("../loader.mjs!" + ref, (err, source, sourceMap, module) => {
		if (err) {
			callback(null, JSON.stringify(`err: ${err && err.message}`));
		} else {
			callback(null, JSON.stringify(`source: ${JSON.parse(source)}`));
		}
	});
};
