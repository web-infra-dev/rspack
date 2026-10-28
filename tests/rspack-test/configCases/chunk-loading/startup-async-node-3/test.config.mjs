import { createRequire } from "node:module";
import path from "node:path";

const require = createRequire(import.meta.url);

async function requireAsync(options) {
	const config = Array.isArray(options) ? options[0] : options;
	const asyncFile = path.join(config.output.path, "async.js");
	delete require.cache[asyncFile];
	return require(asyncFile).MyLib;
}

/** @type {import("../../../..").TConfigCaseConfig} */
export default {
	findBundle: (i, options) => {
		return [];
	},
	async validate(stats, stderr, options) {
		const chunk = await requireAsync(options);
		expect(chunk.result).toBe("123");
	}
};
