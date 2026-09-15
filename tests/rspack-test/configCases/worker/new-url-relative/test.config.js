const fs = require("fs");
const path = require("path");
const { execFileSync } = require("child_process");

const cases = [
	{
		filename: "non-esm.js",
		chunkFilename: "non-esm-worker.bundle.js",
		sharedWorkerChunkFilename: "non-esm-chat.bundle.js",
		runtime: true
	},
	{
		filename: "public-path.js",
		chunkFilename: "public-path-worker.bundle.js",
		sharedWorkerChunkFilename: "public-path-chat.bundle.js",
		url: "/public/public-path-worker.bundle.js"
	},
	{
		filename: "relative-public-path/main.js",
		chunkFilename: "relative-public-path-worker.bundle.js",
		sharedWorkerChunkFilename: "relative-public-path-chat.bundle.js",
		url: "../assets/relative-public-path-worker.bundle.js"
	},
	{
		filename: "worker-public-path.js",
		chunkFilename: "worker-public-path-worker.bundle.js",
		sharedWorkerChunkFilename: "worker-public-path-chat.bundle.js",
		url: "/workers/worker-public-path-worker.bundle.js"
	},
	{
		filename: "relative-worker-public-path/main.js",
		chunkFilename: "relative-worker-public-path-worker.bundle.js",
		sharedWorkerChunkFilename:
			"relative-worker-public-path-chat.bundle.js",
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

			const output = execFileSync(
				process.execPath,
				[
					...(testCase.runtime ? [] : ["--input-type=module"]),
					"--eval",
					`const types = []; const sharedOptions = []; globalThis.Worker = class { constructor(_url, options) { types.push(options?.type); } }; globalThis.SharedWorker = class { constructor(_url, options) { sharedOptions.push(options); } }; ${source}; console.log(JSON.stringify({ types, sharedOptions, evaluationCount: globalThis.__sharedWorkerOptionsEvaluationCount }));`
				],
				{ encoding: "utf8" }
			);
			const result = JSON.parse(output);
			if (testCase.runtime) {
				expect(result).toEqual({
					types: [null, null, null, null],
					sharedOptions: [
						"string-literal",
						"chat",
						"s",
						{ name: "object-literal" },
						"string-variable",
						{ name: "object-variable" },
						"string-expression"
					],
					evaluationCount: 1
				});
			} else {
				expect(result).toEqual({
					types: ["module", "module", "module", "module"],
					sharedOptions: [
						{ name: "string-literal", type: "module" },
						{ name: "chat", type: "module" },
						{ name: "s", type: "module" },
						{ name: "object-literal", type: "module" },
						{ name: "string-variable", type: "module" },
						{ name: "object-variable", type: "module" },
						{ name: "string-expression", type: "module" }
					],
					evaluationCount: 1
				});
			}

			expect(
				fs.existsSync(path.join(outputPath, testCase.chunkFilename))
			).toBe(true);
			expect(
				fs.existsSync(
					path.join(outputPath, testCase.sharedWorkerChunkFilename)
				)
			).toBe(true);
		}
	}
};
