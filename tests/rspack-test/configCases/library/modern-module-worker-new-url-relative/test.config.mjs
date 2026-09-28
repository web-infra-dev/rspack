import fs from "node:fs";
import path from "node:path";

export default {
	findBundle: () => [],
	validate(stats, stderr, options) {
		const config = Array.isArray(options) ? options[0] : options;
		const source = fs.readFileSync(
			path.join(config.output.path, "main.js"),
			"utf-8"
		);
		const workerUrl = source.match(
			/new Worker\([\s\S]*?new URL\("([^"]+)", import\.meta\.url\)/
		)?.[1];

		expect(workerUrl).toBe("./worker.bundle.js");
		expect(
			fs.existsSync(path.join(config.output.path, "worker.bundle.js"))
		).toBe(true);
	}
};
