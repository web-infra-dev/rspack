import { readFileSync } from "node:fs";
import { join } from "node:path";

/** @type {import("../../../..").TConfigCaseConfig} */
export default {
	afterExecute(options) {
		const bundle = readFileSync(join(options.output.path, "bundle0.js"), "utf-8");
		expect(bundle).not.toContain("Automatic publicPath is not supported in this browser");
	},
};
