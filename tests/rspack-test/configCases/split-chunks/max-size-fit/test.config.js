const path = require("path");

/** @type {import("../../../..").TConfigCaseConfig} */
module.exports = {
	writeStatsJson: true,
	findBundle: function (i, options) {
		return ["main-1.js", "main-cde42ecf.js", "main-f5c11e54.js"];
	},
	validate(stats, stderr, options) {
		const config = Array.isArray(options) ? options[0] : options;
		const statsJson = require(path.join(config.output.path, "stats.json"));
		const chunks = new Map();

		for (const c of statsJson.children[0].chunks) {
			chunks.set(c.id, c);
		}

		expect(chunks.size).toBe(3);

		expect(chunks.get("main-1")).toBeDefined();
		expect(chunks.get("main-1").modules.length).toBe(4);
		expect(chunks.get("main-cde42ecf")).toBeDefined();
		expect(chunks.get("main-cde42ecf").modules.length).toBe(1);
		expect(chunks.get("main-cde42ecf").modules[0].name).toBe(
			"./index.js + 5 modules"
		);
		expect(chunks.get("main-f5c11e54")).toBeDefined();
		expect(chunks.get("main-f5c11e54").modules.length).toBe(1);
	}
};
