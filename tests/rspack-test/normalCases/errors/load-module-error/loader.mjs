/** @type {import("@rspack/core").LoaderDefinitionFunction} */
export default function (source) {
	const callback = this.async();
	const ref = JSON.parse(source);
	this.loadModule("./error-loader.mjs!" + ref, (err, source, sourceMap, module) => {
		if (err) {
			callback(err);
		} else {
			callback(null, JSON.stringify(`source: ${JSON.parse(source)}`));
		}
	});
};
