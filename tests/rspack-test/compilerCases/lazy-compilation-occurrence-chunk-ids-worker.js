const fs = require("node:fs");
const os = require("node:os");
const path = require("node:path");
const {
	HotModuleReplacementPlugin,
	lazyCompilationMiddleware
} = require("@rspack/core");

// Occurrence-based chunk ids ("size"/"total-size") are reassigned when lazy
// activation changes the chunk graph. The worker runtime then moves to the
// chunk id that used to belong to the lazy proxy, and HMR has to handle the
// reused id without pairing the old and new chunks.
// https://github.com/web-infra-dev/rspack/issues/16875

let root;
let middleware;

const toPosixPath = identifier => identifier.replaceAll("\\", "/");

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

function hasRealModule(compilation, filename) {
	return [...compilation.modules].some(module => {
		const identifier = module.identifier();
		return (
			!identifier.startsWith("lazy-compilation-proxy") &&
			toPosixPath(identifier).endsWith(`/${filename}`)
		);
	});
}

function createOptions(chunkIds) {
	root = fs.realpathSync(
		fs.mkdtempSync(path.join(os.tmpdir(), "rspack-lazy-occurrence-"))
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
			chunkIds,
			splitChunks: false
		},
		plugins: [new HotModuleReplacementPlugin()]
	};
}

async function run(compiler) {
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
		expect(hasRealModule(initial.stats.compilation, "t1z.js")).toBe(false);

		const moduleId = [...initial.stats.compilation.modules]
			.map(module => module.identifier())
			.find(
				identifier =>
					identifier.includes("lazy-compilation-proxy") &&
					toPosixPath(identifier).endsWith("/t1z.js")
			);
		expect(moduleId).toBeDefined();

		await post(moduleId);
		const activated = await nextBuild();
		expect(activated.error).toBeUndefined();
		expect(hasRealModule(activated.stats.compilation, "t1z.js")).toBe(true);
	} finally {
		await new Promise((resolve, reject) =>
			watching.close(error => (error ? reject(error) : resolve()))
		);
		fs.rmSync(root, { force: true, recursive: true });
	}
}

/** @type {import('@rspack/test-tools').TCompilerCaseConfig[]} */
module.exports = ["size", "total-size"].map(chunkIds => ({
	description: `should activate a lazy module with a worker when chunkIds is "${chunkIds}"`,
	options() {
		return createOptions(chunkIds);
	},
	compiler(_context, compiler) {
		middleware = lazyCompilationMiddleware(compiler);
	},
	build(_context, compiler) {
		return run(compiler);
	}
}));
