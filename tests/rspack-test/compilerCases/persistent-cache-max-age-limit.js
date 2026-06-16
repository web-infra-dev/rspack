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
					maxAge: 1
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

const getCacheEntries = directory =>
	fs
		.readdirSync(directory)
		.filter(name => !name.startsWith("_") && !name.startsWith("."));

const expireVersion = (cacheDirectory, compilerScope, version) => {
	fs.writeFileSync(path.join(cacheDirectory, compilerScope, "_meta"), `${version} 1\n`);
};

/** @type {import('@rspack/test-tools').TCompilerCaseConfig} */
module.exports = {
	description: "should only expire persistent cache versions for the current compiler",
	options(context) {
		return {
			context: context.getSource(),
			entry: "./a"
		};
	},
	async build(context) {
		const cacheDirectory = path.join(
			context.getDist(),
			"persistent-cache-max-age"
		);
		fs.rmSync(cacheDirectory, { recursive: true, force: true });

		await runCompiler(context, cacheDirectory, "app", "v1");
		const [appScope] = getCacheEntries(cacheDirectory);
		const [appV1] = getCacheEntries(path.join(cacheDirectory, appScope));

		await runCompiler(context, cacheDirectory, "worker", "v1");
		const workerScope = getCacheEntries(cacheDirectory).find(
			scope => scope !== appScope
		);
		const [workerV1] = getCacheEntries(path.join(cacheDirectory, workerScope));

		expireVersion(cacheDirectory, appScope, appV1);
		expireVersion(cacheDirectory, workerScope, workerV1);

		await runCompiler(context, cacheDirectory, "app", "v2");
		const appVersions = getCacheEntries(path.join(cacheDirectory, appScope));
		const workerVersions = getCacheEntries(path.join(cacheDirectory, workerScope));

		expect(appVersions).toHaveLength(1);
		expect(appVersions).not.toContain(appV1);
		expect(workerVersions).toEqual([workerV1]);
	}
};
