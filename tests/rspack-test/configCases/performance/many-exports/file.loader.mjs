/** @type {import("@rspack/core").LoaderDefinition} */
export default function () {
	var str = "";
	for (var i = 0; i < 1000; i++) {
		str += `export var a${i} = ${i};\n`;
	}
	return str;
};
