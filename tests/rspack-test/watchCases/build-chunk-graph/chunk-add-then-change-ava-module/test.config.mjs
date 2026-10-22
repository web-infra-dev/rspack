import { checkChunkModules } from "@rspack/test-tools";

export default {
	checkStats(stepName, stats) {
		switch (stepName) {
			case "0":
				checkChunkModules(stats, {
					"dyn-1": ["dyn-1.js", "m.js"],
					shared: ["shared.js"]
				});
				break;
			case "1":
				checkChunkModules(stats, {
					"dyn-1": ["dyn-1.js", "m.js"],
					"dyn-2": ["dyn-2.js"],
					shared: ["shared.js", "m.js"]
				});
				break;
			case "2":
				checkChunkModules(stats, {
					"dyn-1": ["dyn-1.js", "m.js"],
					"dyn-2": ["dyn-2.js", "m.js"],
					shared: ["shared.js"]
				});
				break;
			default:
				throw "no have more step";
		}

		return true;
	}
};
