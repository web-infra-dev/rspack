import fs from "node:fs";
import path from "node:path";

export default {
	findBundle(index, options) {
		const files = fs.readdirSync(options.output.path);
		const initialCssFile = files.find(file =>
			file.startsWith("initial-style.")
		);
		expect(initialCssFile).toBeTruthy();

		const runtime = fs.readFileSync(
			path.join(options.output.path, "runtime.js"),
			"utf-8"
		);
		expect(runtime).toContain('"initial-style" == chunkId');
		const contentHash = initialCssFile.split(".")[1];
		expect(runtime).toContain(contentHash);

		return [];
	}
};
