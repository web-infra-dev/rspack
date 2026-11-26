const fs = require("node:fs");
const os = require("node:os");
const path = require("node:path");
const {
	HotModuleReplacementPlugin,
	lazyCompilationMiddleware
} = require("@rspack/core");

// Two lazy chunks whose compact-hashed ids share a hash prefix. Activating the
// first one adds the worker runtime to the second one's chunk. That used to
// reorder the chunks during id assignment, swap their short ids between
// rebuilds, and make HMR pair the wrong old/new chunks, panicking during code
// generation for the lazy proxy.
// https://github.com/web-infra-dev/rspack/issues/16875

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

function proxyChunkIds(compilation) {
	const { chunkGraph } = compilation;
	const ids = new Map();
	for (const chunk of compilation.chunks) {
		for (const module of chunkGraph.getChunkModulesIterable(chunk)) {
			const identifier = module.identifier();
			if (identifier.startsWith("lazy-compilation-proxy")) {
				ids.set(identifier, String(chunk.id));
			}
		}
	}
	return ids;
}

/** @type {import('@rspack/test-tools').TCompilerCaseConfig[]} */
module.exports = [
	{
		description:
			"should keep compact-hashed chunk ids stable when lazy activation changes a chunk runtime",
		options() {
			root = fs.realpathSync(
				fs.mkdtempSync(path.join(os.tmpdir(), "rspack-lazy-compact-hashed-"))
			);
			write(
				"entry.js",
				'globalThis.load=()=>import("./t1z.js");globalThis.other=()=>import("./t8d.js");'
			);
			write(
				"t1z.js",
				'export const spawn=()=>new Worker(new URL("./worker.js",import.meta.url));'
			);
			write("worker.js", 'globalThis.other=()=>import("./t8d.js");');
			write("t8d.js", "export const value = 1;");

			return {
				context: root,
				mode: "development",
				target: "web",
				devtool: false,
				entry: "./entry.js",
				output: { filename: "main.js", chunkFilename: "[id].js" },
				lazyCompilation: { entries: false, imports: true },
				optimization: {
					minimize: false,
					chunkIds: "compact-hashed",
					splitChunks: false
				},
				plugins: [new HotModuleReplacementPlugin()]
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
				const initialIds = proxyChunkIds(initial.stats.compilation);
				const collidingIds = [...initialIds]
					.filter(([identifier]) => /\/(t1z|t8d)\.js$/.test(identifier))
					.map(([, id]) => id);
				expect(collidingIds).toHaveLength(2);
				expect(collidingIds[0][0]).toBe(collidingIds[1][0]);

				const moduleId = [...initial.stats.compilation.modules]
					.map(module => module.identifier())
					.find(
						identifier =>
							identifier.includes("lazy-compilation-proxy") &&
							identifier.endsWith("/t1z.js")
					);
				expect(moduleId).toBeDefined();

				await post(moduleId);
				const activated = await nextBuild();
				expect(activated.error).toBeUndefined();

				const activatedIds = proxyChunkIds(activated.stats.compilation);
				for (const [identifier, id] of initialIds) {
					if (identifier.endsWith("/t8d.js")) {
						expect(activatedIds.get(identifier)).toBe(id);
					}
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
