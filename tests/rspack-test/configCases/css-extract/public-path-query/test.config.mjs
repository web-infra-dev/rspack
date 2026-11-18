import fs from "node:fs";
import path from "node:path";

let outputPath;

export default {
	findBundle(index, options) {
		outputPath = options.output.path;
		return ["main.js"];
	},
	// The public path routes every asset through one URL, told apart by its query.
	resourceLoader(url) {
		const file = new URL(url).searchParams.get("file");
		if (file) return fs.readFileSync(path.join(outputPath, file));
	}
};
