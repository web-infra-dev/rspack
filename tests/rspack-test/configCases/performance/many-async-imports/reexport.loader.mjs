/** @type {import("@rspack/core").LoaderDefinition} */
export default function () {
	var str = "export default Promise.all([\n";
	for (var i = 0; i < 6; i++) {
		for (var j = 0; j < 2; j++) {
			str += `import("./reexport.loader.mjs!?${i}"),\n`;
		}
	}
	str += "]);";
	return str;
};
