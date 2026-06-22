const path = require("path");

async function requireAsync(options) {
	const config = Array.isArray(options) ? options[0] : options;
	const asyncFile = path.join(config.output.path, "async.js");
	delete require.cache[asyncFile];
	return require(asyncFile).MyLib;
}

/** @type {import("../../../..").TConfigCaseConfig} */
module.exports = {
	findBundle: (i, options) => {
		return [];
	},
	async validate(stats, stderr, options) {
		const chunk = await requireAsync(options);
		expect(chunk.result).toBe("1");
	}
};
