const fs = require("fs");
const path = require("path");

module.exports = {
	findBundle: () => [],
	validate(stats, stderr, options) {
		const config = Array.isArray(options) ? options[0] : options;
		const source = fs.readFileSync(
			path.join(config.output.path, "js/main.js"),
			"utf-8"
		);
		const workerUrl = source.match(
			/new Worker\([\s\S]*?new URL\("([^"]+)", import\.meta\.url\)/
		)?.[1];

		expect(workerUrl).toBe(path.posix.join("..", "worker.bundle.js"));
		expect(
			fs.existsSync(path.join(config.output.path, "worker.bundle.js"))
		).toBe(true);
	}
};
