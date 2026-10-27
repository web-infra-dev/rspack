import path from "node:path";
import { pathToFileURL } from "node:url";
/** @type {import("@rspack/core").LoaderDefinition} */
export default function () {
	const sources = [
		pathToFileURL(path.join(import.meta.dirname, "folder", "file-url.txt")).href,
		pathToFileURL(path.join(import.meta.dirname, "my folder", "encoded.txt")).href,
		pathToFileURL(path.join(import.meta.dirname, "..", "sibling", "outside.txt")).href,
		`webpack://${path.join(import.meta.dirname, "folder", "webpack-url.txt")}`,
		"file://server/share/remote.txt",
		"webpack://lib/./src/namespaced.txt"
	];
	this.callback(null, "module.exports = 'ok';", {
		version: 3,
		sources,
		sourcesContent: sources.map(() => "Test"),
		names: [],
		mappings: "AAAA;AACA;AACA;AACA;AACA;AACA"
	});
};
