const fs = require("node:fs");
const path = require("node:path");
const rspack = require("@rspack/core");

const runCompiler = (context, cacheDirectory, name, version) =>
	new Promise((resolve, reject) => {
		const compiler = rspack({
			name,
			context: context.getSource(),
			entry: "./a",
			mode: "development",
			output: {
				path: path.join(context.getDist(), `${name}-${version}`)
			},
			cache: {
				type: "persistent",
				version,
				storage: {
					type: "filesystem",
					directory: cacheDirectory,
					maxVersions: 1
				}
			}
		});

		compiler.run(error => {
			compiler.close(closeError => {
				const finalError = error || closeError;
				if (finalError) {
					reject(finalError);
					return;
				}
				resolve();
			});
		});
	});

const getVersions = directory =>
	fs
		.readdirSync(directory)
		.filter(name => !name.startsWith("_") && !name.startsWith("."));

/** @type {import('@rspack/test-tools').TCompilerCaseConfig} */
module.exports = {
	description: "should limit persistent cache versions per compiler",
	options(context) {
		return {
			context: context.getSource(),
			entry: "./a"
		};
	},
	async build(context) {
		const cacheDirectory = path.join(
			context.getDist(),
			"persistent-cache-max-versions"
		);
		fs.rmSync(cacheDirectory, { recursive: true, force: true });

		await runCompiler(context, cacheDirectory, "app", "v1");
		const [appV1] = getVersions(cacheDirectory);

		await runCompiler(context, cacheDirectory, "worker", "v1");
		const workerV1 = getVersions(cacheDirectory).find(
			version => version !== appV1
		);

		await runCompiler(context, cacheDirectory, "app", "v2");
		const retainedVersions = getVersions(cacheDirectory);

		expect(retainedVersions).toHaveLength(2);
		expect(retainedVersions).toContain(workerV1);
		expect(retainedVersions).not.toContain(appV1);
	}
};
