import fs from "node:fs";
import path from "node:path";

/** @type {import("@rspack/core").LoaderDefinition<{ f(): any }>} */
export default function(_) {
	// return the would-be output from SASS without needing the compiler as a dependency
	const transformed = fs.readFileSync(path.join(import.meta.dirname, "data/asset.css"), { encoding: "utf8" });
	const sourceMap = fs.readFileSync(path.join(import.meta.dirname, "data/asset.css.map"), { encoding: "utf8" });

	this.callback(null, transformed, JSON.parse(sourceMap));
}
