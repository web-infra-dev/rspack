const { readAsset } = require("./_read-asset.cjs");
const fs = require("node:fs");
const os = require("node:os");
const path = require("node:path");
const { lazyCompilationMiddleware } = require("@rspack/core");

// A lazy entry removed in the compilation that consumes its first activation,
// while a new import() of the same module is added, keeps its inactive proxy.
// The client then re-sends the activation, which must still activate it.
// https://github.com/web-infra-dev/rspack/pull/15134#discussion_r4227747529

let root;
let middleware;
let entries = { main: "./src/main.js", feature: "./src/feature.js" };

function write(file, content) {
	const target = path.join(root, file);
	fs.mkdirSync(path.dirname(target), { recursive: true });
	fs.writeFileSync(target, content);
}

function post(moduleId) {
	return new Promise((resolve, reject) => {
		middleware(
			{ body: [moduleId], method: "POST", url: "/_rspack/lazy/trigger" },
			{
				end: resolve,
				write() {},
				writeHead(status) {
					expect(status).toBe(200);
				}
			},
			reject
		).catch(reject);
	});
}

function emitsPayload(compilation) {
	return Object.keys(compilation.assets).some(name =>
		readAsset(compilation, name).includes("FEATURE_PAYLOAD")
	);
}

/** @type {import('@rspack/test-tools').TCompilerCaseConfig[]} */
module.exports = [
	{
		description:
			"should activate a lazy entry that becomes an import() target before its first activation build",
		options() {
			root = fs.mkdtempSync(path.join(os.tmpdir(), "rspack-lazy-entry-to-import-"));
			write("src/main.js", "globalThis.main = true;\n");
			write("src/main-import.js", "import('./feature.js');\n");
			write("src/feature.js", "globalThis.feature = 'FEATURE_PAYLOAD';\n");

			return {
				context: root,
				mode: "development",
				target: "web",
				devtool: false,
				entry: async () => entries,
				lazyCompilation: { entries: true, imports: true, test: /feature\.js$/ },
				output: { chunkFilename: "[name].js" },
				optimization: { minimize: false, moduleIds: "named", chunkIds: "named" }
			};
		},
		compiler(_context, compiler) {
			middleware = lazyCompilationMiddleware(compiler);
		},
		async build(_context, compiler) {
			const builds = [];
			const waiters = [];
			const nextBuild = () =>
				builds.length > 0
					? Promise.resolve(builds.shift())
					: new Promise((resolve, reject) => {
							const timeout = setTimeout(
								() => reject(new Error("Timed out waiting for a rebuild")),
								10000
							);
							waiters.push(value => {
								clearTimeout(timeout);
								resolve(value);
							});
						});

			const watching = compiler.watch({}, (error, stats) => {
				const value = {
					error:
						error ??
						(stats?.hasErrors()
							? new Error(stats.toString({ all: false, errors: true }))
							: undefined),
					stats
				};
				(waiters.shift() ?? (build => builds.push(build)))(value);
			});

			try {
				const initial = await nextBuild();
				expect(initial.error).toBeUndefined();
				expect(emitsPayload(initial.stats.compilation)).toBe(false);

				const moduleId = [...initial.stats.compilation.modules]
					.map(module => module.identifier())
					.find(identifier => identifier.includes("lazy-compilation-proxy"));
				expect(moduleId).toBeDefined();

				entries = { main: "./src/main-import.js" };
				await post(moduleId);
				const switched = await nextBuild();
				expect(switched.error).toBeUndefined();

				if (!emitsPayload(switched.stats.compilation)) {
					await post(moduleId);
					const retried = await nextBuild();
					expect(retried.error).toBeUndefined();
					expect(emitsPayload(retried.stats.compilation)).toBe(true);
				}
			} finally {
				await new Promise((resolve, reject) =>
					watching.close(error => (error ? reject(error) : resolve()))
				);
				fs.rmSync(root, { force: true, recursive: true });
			}
		}
	}
];
