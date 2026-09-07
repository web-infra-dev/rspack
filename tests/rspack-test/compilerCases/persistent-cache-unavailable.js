const fs = require("node:fs");
const path = require("node:path");
const { createFsFromVolume, Volume } = require("memfs");

const CACHE_LOGGER = "rspack.cache.IdleFileCache";

async function run(compiler) {
	const stats = await new Promise((resolve, reject) => {
		compiler.run((error, stats) => (error ? reject(error) : resolve(stats)));
	});
	expect(stats.hasErrors()).toBe(false);
	expect(stats.hasWarnings()).toBe(false);
}

/** @type {import('@rspack/test-tools').TCompilerCaseConfig} */
module.exports = {
	description: "should disable the file cache after a commit fails and keep rebuilding",
	options(context) {
		const cacheLocation = context.getDist("cache");
		context.setValue("cacheLocation", cacheLocation);
		return {
			context: context.getSource(),
			entry: "./a",
			mode: "production",
			optimization: { minimize: false },
			output: { path: context.getDist("output"), pathinfo: true },
			experiments: { newCache: true },
			cache: {
				type: "persistent",
				storage: { type: "filesystem", location: cacheLocation }
			},
			infrastructureLogging: { level: "none" }
		};
	},
	compiler(context, compiler) {
		compiler.outputFileSystem = createFsFromVolume(new Volume());
		const warnings = [];
		context.setValue("cacheWarnings", warnings);
		context.setValue(
			"unavailable",
			new Promise(resolve => {
				compiler.hooks.infrastructureLog.tap(
					"UnavailableCacheTest",
					(name, type, args) => {
						if (name === CACHE_LOGGER && type === "warn") {
							warnings.push(args[0]);
							if (args[0].includes("Filesystem cache unavailable for this session")) {
								resolve();
							}
						}
					}
				);
			})
		);
		compiler.hooks.afterCompile.tap("UnavailableCacheTest", () => {
			if (context.getValue("blocker")) return;
			const cacheLocation = context.getValue("cacheLocation");
			context.setValue("current", fs.readFileSync(path.join(cacheLocation, "CURRENT")));
			// A directory in place of CURRENT.next makes the first idle commit fail.
			const blocker = path.join(cacheLocation, "CURRENT.next");
			fs.mkdirSync(blocker);
			context.setValue("blocker", blocker);
		});
	},
	async build(context, compiler) {
		await run(compiler);
		await context.getValue("unavailable");
		// Removing the fault must not re-enable persistence in the same session.
		fs.rmdirSync(context.getValue("blocker"));
		await run(compiler);
		await run(compiler);
		await context.closeCompiler();
		await context.closeCompiler();
		const warnings = context.getValue("cacheWarnings");
		expect(warnings).toHaveLength(1);
		expect(warnings[0]).toContain("Committing CURRENT file failed");
		expect(
			fs.readFileSync(path.join(context.getValue("cacheLocation"), "CURRENT"))
		).toEqual(context.getValue("current"));
	}
};
