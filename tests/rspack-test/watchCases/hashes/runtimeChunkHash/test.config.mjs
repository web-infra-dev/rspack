import fs from "node:fs";
const runned = new Set();

export default {
	findBundle(i, config) {
		const main = fs
			.readdirSync(config.output.path)
			.find(i => i.includes("main.") && !runned.has(i));
		runned.add(main);
		return [main];
	}
};
