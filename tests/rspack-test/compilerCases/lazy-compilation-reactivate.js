const fs = require("node:fs");
const os = require("node:os");
const path = require("node:path");
const { lazyCompilationMiddleware } = require("@rspack/core");

// After an HMR apply the client re-sends its whole active set. Re-reporting a
// module that is already active must not schedule another rebuild, otherwise
// activation and HMR keep triggering each other.
// https://github.com/web-infra-dev/rspack/issues/15062

const SETTLE = 1000;

let root;
let middleware;

function write(filename, content) {
	const file = path.join(root, filename);
	fs.mkdirSync(path.dirname(file), { recursive: true });
	fs.writeFileSync(file, content);
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

/** @type {import('@rspack/test-tools').TCompilerCaseConfig[]} */
module.exports = [
	{
		description:
			"should not rebuild when an already activated module is reported again",
		options() {
			root = fs.mkdtempSync(path.join(os.tmpdir(), "rspack-lazy-reactivate-"));
			write("src/main.js", "import('./dyn.js');\n");
			write("src/dyn.js", "globalThis.dyn = 'DYN_PAYLOAD';\n");

			return {
				context: root,
				mode: "development",
				target: "web",
				devtool: false,
				entry: { main: "./src/main.js" },
				lazyCompilation: { entries: false, imports: true },
				output: { filename: "main.js", chunkFilename: "[name].js" },
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

				const moduleId = [...initial.stats.compilation.modules]
					.map(module => module.identifier())
					.find(identifier => identifier.includes("lazy-compilation-proxy"));
				expect(moduleId).toBeDefined();

				// First activation compiles the module behind the proxy.
				await post(moduleId);
				const activated = await nextBuild();
				expect(activated.error).toBeUndefined();
				const { compilation } = activated.stats;
				expect(
					Object.keys(compilation.assets).some(name =>
						compilation
							.getAsset(name)
							.source.source()
							.toString()
							.includes("DYN_PAYLOAD")
					)
				).toBe(true);

				// Re-reporting the same module must be a no-op.
				await post(moduleId);
				const settled = await Promise.race([
					nextBuild().then(() => "rebuilt"),
					new Promise(resolve => setTimeout(() => resolve("idle"), SETTLE))
				]);
				expect(settled).toBe("idle");
			} finally {
				await new Promise((resolve, reject) =>
					watching.close(error => (error ? reject(error) : resolve()))
				);
				fs.rmSync(root, { force: true, recursive: true });
			}
		}
	}
];
