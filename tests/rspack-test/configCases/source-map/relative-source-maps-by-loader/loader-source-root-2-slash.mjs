import path from "node:path";
/** @type {import("@rspack/core").LoaderDefinition} */
export default function () {
	this.callback(null, "module.exports = 'ok';", {
		version: 3,
		file: "/should/be/removed",
		sourceRoot: path.join(import.meta.dirname, "folder") + "/",
		sources: ["/test4.txt"],
		sourcesContent: ["Test"],
		names: [],
		mappings: "AAAA"
	});
};
