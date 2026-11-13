import fs from "node:fs";
import path from "node:path";

/** @type {import("../../../..").TConfigCaseConfig} */
export default {
	afterExecute(options) {
		const source = fs
			.readFileSync(path.join(options.output.path, options.output.filename), "utf-8")
			.replace(/"[a-f0-9]+"/g, '"HASH"');
		for (const extension of ["js", "css"]) {
			expect(source).toContain(
				`return chunkId + "." + "HASH" + ".${extension}"`
			);
		}
	}
};
