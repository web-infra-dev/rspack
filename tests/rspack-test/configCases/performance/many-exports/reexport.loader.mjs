/** @type {import("@rspack/core").LoaderDefinition} */
export default function () {
	var str = 'import * as i from "./file.loader.mjs!";\n';
	str += "var sum = 0;\n";
	for (var i = 0; i < 1000; i++) {
		str += `sum += i.a${i};\n`;
	}
	str += "export default sum;\n";
	return str;
};
