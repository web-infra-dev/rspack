const fs = require("fs");
const path = require("path");

const cases = [
	{
		filename: "non-esm.js",
		chunkFilename: "non-esm-worker.bundle.js",
		runtime: true
	},
	{
		filename: "public-path.js",
		chunkFilename: "public-path-worker.bundle.js",
		url: "/public/public-path-worker.bundle.js"
	},
	{
		filename: "relative-public-path/main.js",
		chunkFilename: "relative-public-path-worker.bundle.js",
		url: "../assets/relative-public-path-worker.bundle.js"
	},
	{
		filename: "worker-public-path.js",
		chunkFilename: "worker-public-path-worker.bundle.js",
		url: "/workers/worker-public-path-worker.bundle.js"
	},
	{
		filename: "relative-worker-public-path/main.js",
		chunkFilename: "relative-worker-public-path-worker.bundle.js",
		url: "../workers/relative-worker-public-path-worker.bundle.js"
	}
];

module.exports = {
	findBundle: () => [],
	validate(stats, stderr, options) {
		const configs = Array.isArray(options) ? options : [options];

		for (const [index, testCase] of cases.entries()) {
			const outputPath = configs[index].output.path;
			const source = fs.readFileSync(
				path.join(outputPath, testCase.filename),
				"utf-8"
			);
			const workerUrl = source.match(
				/new Worker\([\s\S]*?new URL\("([^"]+)", import\.meta\.url\)/
			)?.[1];

			if (testCase.runtime) {
				expect(workerUrl).toBeUndefined();
				expect(source).toMatch(
					/\/\* worker import \*\/[\w$]+\.p \+ [\w$]+\.u\(/
				);
			} else {
				expect(workerUrl).toBe(testCase.url);
			}

			expect(
				fs.existsSync(path.join(outputPath, testCase.chunkFilename))
			).toBe(true);
		}
	}
};
