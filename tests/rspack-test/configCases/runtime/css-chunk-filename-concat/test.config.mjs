import fs from "node:fs";
import path from "node:path";

/** @type {import("../../../..").TConfigCaseConfig} */
export default {
	afterExecute(options) {
		const source = fs.readFileSync(
			path.join(options.output.path, "bundle0.js"),
			"utf-8"
		);
		const match = source.match(/return chunkId \+ "(\.[a-f0-9]+\.css)";/);
		expect(match).not.toBeNull();
		expect(
			fs.existsSync(path.join(options.output.path, `12345${match[1]}`))
		).toBe(true);
	}
};
