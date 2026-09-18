const fs = require("node:fs");
const path = require("node:path");

const CASE_NAME = "interned-path-native-separators";
const INITIAL_VALUE = "initial value";
const UPDATED_VALUE = "updated value from watch";

function write(file, content) {
	fs.mkdirSync(path.dirname(file), { recursive: true });
	fs.writeFileSync(file, content);
}

const toSlash = file => file.replace(/\\/g, "/");

function watchWithSlashDependency(context, compiler, valueFile, outputFile) {
	return new Promise((resolve, reject) => {
		let buildCount = 0;
		let written = false;
		let changeTimer;
		let watching;
		let settled = false;

		const timeout = setTimeout(
			() => finish(new Error("Timed out waiting for the watcher to rebuild")),
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

		// A JS plugin or loader handing rspack the `/` spelling of a file before the resolver
		// interns its native spelling (rspress' page data module does exactly this).
		compiler.hooks.thisCompilation.tap(CASE_NAME, compilation => {
			if (buildCount === 0) {
				compilation.fileDependencies.add(toSlash(valueFile));
			}
		});

		watching = compiler.watch(
			{
				aggregateTimeout: 50,
				ignored: []
			},
			(error, stats) => {
				if (settled) return;
				if (error) return finish(error);
				if (stats.hasErrors()) {
					return finish(
						new Error(
							stats.toString({ all: false, errors: true, errorDetails: true })
						)
					);
				}

				buildCount += 1;
				const dependencies = Array.from(stats.compilation.fileDependencies);
				const valueDependencies = dependencies.filter(
					dependency =>
						toSlash(dependency).toLowerCase() ===
						toSlash(valueFile).toLowerCase()
				);

				if (buildCount === 1) {
					context.setValue("initialOutput", fs.readFileSync(outputFile, "utf8"));
					context.setValue("valueDependencies", valueDependencies);
					changeTimer = setTimeout(() => {
						written = true;
						write(valueFile, `export default ${JSON.stringify(UPDATED_VALUE)};\n`);
					}, process.platform === "win32" ? 500 : 100);
					return;
				}

				// A rebuild may fire for the freshly written sources before the change above;
				// only the build that picked up the change settles the case.
				const output = fs.readFileSync(outputFile, "utf8");
				if (!written || !output.includes(UPDATED_VALUE)) return;
				context.setValue("buildCount", buildCount);
				context.setValue("updatedOutput", output);
				finish();
			}
		);
	});
}

module.exports = {
	description:
		"should keep watching a file whose `/` spelling was interned before its native spelling",
	options(context) {
		const root = context.getDist(CASE_NAME);
		const sourceDirectory = path.join(root, "src");
		const outputDirectory = path.join(root, "dist");
		const valueFile = path.join(sourceDirectory, "value.js");

		fs.rmSync(root, { recursive: true, force: true });
		write(
			path.join(sourceDirectory, "index.js"),
			'import value from "./value.js";\nconsole.log(value);\n'
		);
		write(valueFile, `export default ${JSON.stringify(INITIAL_VALUE)};\n`);

		context.setValue("valueFile", valueFile);
		context.setValue("outputFile", path.join(outputDirectory, "main.js"));
		return {
			context: sourceDirectory,
			mode: "development",
			devtool: false,
			target: "node",
			entry: "./index.js",
			output: {
				path: outputDirectory,
				filename: "main.js",
				clean: true
			}
		};
	},
	compiler(_context, compiler) {
		compiler.outputFileSystem = fs;
	},
	async build(context, compiler) {
		await watchWithSlashDependency(
			context,
			compiler,
			context.getValue("valueFile"),
			context.getValue("outputFile")
		);
	},
	check({ context }) {
		// Only the native spelling may reach `fileDependencies`: watchpack keys watchers by the
		// exact string and never matches a `/` spelling on Windows.
		expect(context.getValue("valueDependencies")).toEqual([
			context.getValue("valueFile")
		]);
		expect(context.getValue("buildCount")).toBeGreaterThanOrEqual(2);
		expect(context.getValue("initialOutput")).toContain(INITIAL_VALUE);
		expect(context.getValue("updatedOutput")).toContain(UPDATED_VALUE);
	}
};
