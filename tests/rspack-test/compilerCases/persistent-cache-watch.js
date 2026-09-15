const { execFileSync } = require("node:child_process");
const fs = require("node:fs");
const path = require("node:path");

const CASE_NAME = "persistent-cache-watch";
const INITIAL_VALUE = "initial value";
const UPDATED_VALUE = "updated value from watch";

const WARM_CACHE_SCRIPT = String.raw`
const rspack = require(process.argv[1]);
const options = JSON.parse(process.argv[2]);
const compiler = rspack(options);

compiler.run((error, stats) => {
	const statsError =
		!error && stats && stats.hasErrors()
			? new Error(stats.toString({ all: false, errors: true, errorDetails: true }))
			: null;

	compiler.close(closeError => {
		const finalError = error || statsError || closeError;
		if (finalError) {
			console.error(finalError);
			process.exitCode = 1;
		}
	});
});
`;

function write(file, content) {
	fs.mkdirSync(path.dirname(file), { recursive: true });
	fs.writeFileSync(file, content);
}

function warmPersistentCache(options) {
	execFileSync(
		process.execPath,
		[
			"-e",
			WARM_CACHE_SCRIPT,
			require.resolve("@rspack/core"),
			JSON.stringify(options)
		],
		{
			cwd: options.context,
			stdio: "pipe",
			windowsHide: true
		}
	);
}

function watchAfterCacheRestore(context, compiler, valueFile, outputFile) {
	return new Promise((resolve, reject) => {
		let buildCount = 0;
		let changeTimer;
		let watching;
		let settled = false;

		const timeout = setTimeout(
			() => finish(new Error("Timed out waiting for the restored watcher to rebuild")),
			process.platform === "win32" ? 15_000 : 10_000
		);

		const finish = error => {
			if (settled) return;
			settled = true;
			clearTimeout(timeout);
			clearTimeout(changeTimer);

			const done = closeError => {
				const finalError = error || closeError;
				if (finalError) {
					reject(finalError);
				} else {
					resolve();
				}
			};

			if (watching) {
				watching.close(done);
			} else {
				done();
			}
		};

		watching = compiler.watch(
			{
				aggregateTimeout: 50,
				ignored: []
			},
			(error, stats) => {
				try {
					if (error) return finish(error);
					if (!stats) return finish(new Error("Watch build returned no stats"));
					if (stats.hasErrors()) {
						return finish(
							new Error(
								stats.toString({ all: false, errors: true, errorDetails: true })
							)
						);
					}

					buildCount++;
					if (buildCount === 1) {
						context.setValue(
							"initialCacheLog",
							stats.toString({
								all: false,
								colors: false,
								logging: false,
								loggingDebug: /^rspack\.persistentCache$/
							})
						);
						context.setValue(
							"restoredValueDependency",
							[...stats.compilation.fileDependencies].find(
								dependency => path.basename(dependency) === "value.js"
							)
						);
						context.setValue("initialOutput", fs.readFileSync(outputFile, "utf8"));

						// Watchpack is ready after the initial callback, but Windows may need a little
						// longer to install the native directory watcher under concurrent CI load.
						changeTimer = setTimeout(
							() => write(valueFile, `export default ${JSON.stringify(UPDATED_VALUE)};\n`),
							process.platform === "win32" ? 500 : 100
						);
						return;
					}

					context.setValue("buildCount", buildCount);
					context.setValue("updatedOutput", fs.readFileSync(outputFile, "utf8"));
					finish();
				} catch (callbackError) {
					finish(callbackError);
				}
			}
		);
	});
}

/** @type {import("@rspack/test-tools").TCompilerCaseConfig} */
module.exports = {
	description:
		"should detect native file changes after restoring a persistent cache",
	options(context) {
		const root = context.getDist(CASE_NAME);
		const sourceDirectory = path.join(root, "src");
		const outputDirectory = path.join(root, "dist");
		const cacheDirectory = path.join(root, "cache");
		const valueFile = path.join(sourceDirectory, "value.js");

		fs.rmSync(root, { recursive: true, force: true });
		write(
			path.join(sourceDirectory, "index.js"),
			'import value from "./value.js";\nconsole.log(value);\n'
		);
		write(valueFile, `export default ${JSON.stringify(INITIAL_VALUE)};\n`);

		const options = {
			context: sourceDirectory,
			mode: "development",
			devtool: false,
			target: "node",
			entry: "./index.js",
			cache: {
				type: "persistent",
				portable: false,
				version: CASE_NAME,
				storage: {
					type: "filesystem",
					directory: cacheDirectory
				}
			},
			output: {
				path: outputDirectory,
				filename: "main.js",
				clean: true
			}
		};

		// Use another process so no native path from the warm build can remain in the
		// process-wide path interner before the persistent cache is restored.
		warmPersistentCache(options);
		context.setValue("valueFile", valueFile);
		context.setValue("outputFile", path.join(outputDirectory, "main.js"));
		return options;
	},
	compiler(_context, compiler) {
		compiler.outputFileSystem = fs;
	},
	async build(context, compiler) {
		await watchAfterCacheRestore(
			context,
			compiler,
			context.getValue("valueFile"),
			context.getValue("outputFile")
		);
	},
	check({ context }) {
		expect(context.getValue("initialCacheLog")).toContain(
			"make persistent cache recovery succeeded"
		);
		expect(context.getValue("restoredValueDependency")).toBe(
			context.getValue("valueFile")
		);
		expect(context.getValue("buildCount")).toBe(2);
		expect(context.getValue("initialOutput")).toContain(INITIAL_VALUE);
		expect(context.getValue("updatedOutput")).toContain(UPDATED_VALUE);
	}
};
