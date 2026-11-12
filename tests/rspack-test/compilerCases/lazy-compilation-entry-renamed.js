const fs = require("node:fs");
const os = require("node:os");
const path = require("node:path");
const { lazyCompilationMiddleware } = require("@rspack/core");

// A dynamic entry whose options change in the same compilation that consumes
// its first activation gets a new entry dependency. The proxy still points to
// the old one at `make` time, but the source is the same, so the proxy must
// end up active once the old entry dependency is dropped.
// https://github.com/web-infra-dev/rspack/pull/15134#discussion_r4226930763

let root;
let middleware;
let filename = "a.js";

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
		compilation.getAsset(name).source.source().toString().includes("MAIN_PAYLOAD")
	);
}

/** @type {import('@rspack/test-tools').TCompilerCaseConfig[]} */
module.exports = [
	{
		description:
			"should activate a lazy entry whose options change before its first activation build",
		options() {
			root = fs.mkdtempSync(path.join(os.tmpdir(), "rspack-lazy-entry-renamed-"));
			write("src/main.js", "globalThis.main = 'MAIN_PAYLOAD';\n");

			return {
				context: root,
				mode: "development",
				target: "web",
				devtool: false,
				entry: async () => ({ main: { import: "./src/main.js", filename } }),
				lazyCompilation: { entries: true, imports: false },
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

				filename = "b.js";
				await post(moduleId);
				const activated = await nextBuild();
				expect(activated.error).toBeUndefined();
				expect(emitsPayload(activated.stats.compilation)).toBe(true);
			} finally {
				await new Promise((resolve, reject) =>
					watching.close(error => (error ? reject(error) : resolve()))
				);
				fs.rmSync(root, { force: true, recursive: true });
			}
		}
	}
];
