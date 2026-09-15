const fs = require("node:fs");
const path = require("node:path");

let root;
let traceFile;
let issuerSnapshots;
let savedModuleCounts;

const makeOnlyIncremental = {
	silent: true,
	buildModuleGraph: true,
	finishModules: false,
	optimizeDependencies: false,
	buildChunkGraph: false,
	optimizeChunkModules: false,
	moduleIds: false,
	chunkIds: false,
	modulesHashes: false,
	modulesCodegen: false,
	modulesRuntimeRequirements: false,
	chunksRuntimeRequirements: false,
	chunksHashes: false,
	chunkAsset: false,
	emitAssets: false
};

function write(file, content) {
	fs.mkdirSync(path.dirname(file), { recursive: true });
	fs.writeFileSync(file, content);
}

function run(compiler, changes = {}) {
	return new Promise((resolve, reject) => {
		compiler.run((error, stats) => {
			if (error) return reject(error);
			if (stats.hasErrors()) {
				return reject(new Error(stats.toString({ all: false, errors: true })));
			}
			resolve();
		}, changes);
	});
}

function flushCache(compiler) {
	return new Promise((resolve, reject) => {
		compiler.cache.endIdle(error => (error ? reject(error) : resolve()));
	});
}

/** @type {import('@rspack/test-tools').TCompilerCaseConfig} */
module.exports = {
	description: "should not persist stale issuer updates across rebuilds",
	options(context) {
		root = context.getDist("persistent-cache-issuer-updates");
		traceFile = path.join(root, "rspack.log");
		issuerSnapshots = [];
		savedModuleCounts = [];

		fs.rmSync(root, { recursive: true, force: true });
		write(
			path.join(root, "src/index.js"),
			'import "./a.js";\nimport "./b.js";\nimport "./unrelated.js";\n'
		);
		write(path.join(root, "src/a.js"), 'import "./shared.js";\n');
		write(path.join(root, "src/b.js"), 'import "./shared.js";\n');
		write(path.join(root, "src/shared.js"), "globalThis.shared = true;\n");
		write(
			path.join(root, "src/unrelated.js"),
			'globalThis.unrelated = "initial";\n'
		);

		return {
			context: path.join(root, "src"),
			mode: "development",
			devtool: false,
			entry: "./index.js",
			cache: {
				type: "persistent",
				version: "issuer-update-repro",
				storage: {
					type: "filesystem",
					directory: path.join(root, "cache")
				}
			},
			incremental: makeOnlyIncremental,
			plugins: [
				{
					apply(compiler) {
						compiler.hooks.done.tap("CaptureSharedIssuer", stats => {
							const shared = [...stats.compilation.modules].find(module =>
								module.resource === path.join(root, "src/shared.js")
							);
							const issuer =
								shared && stats.compilation.moduleGraph.getIssuer(shared);
							issuerSnapshots.push(issuer?.resource ?? null);
						});
					}
				}
			],
			output: {
				path: path.join(root, "dist"),
				filename: "main.js",
				clean: true
			}
		};
	},
	compiler(_context, compiler) {
		compiler.outputFileSystem = fs;
	},
	async build(_context, compiler) {
		const entryFile = path.join(root, "src/index.js");
		const unrelatedFile = path.join(root, "src/unrelated.js");

		await compiler.rspack.experiments.globalTrace.register(
			"rspack_core=debug",
			"logger",
			traceFile
		);

		try {
			await run(compiler);
			await flushCache(compiler);

			const initialIssuer = path.basename(issuerSnapshots[0], ".js");
			const remainingIssuer = initialIssuer === "a" ? "b" : "a";
			write(
				entryFile,
				`import "./${remainingIssuer}.js";\nimport "./unrelated.js";\n`
			);
			await run(compiler, { modifiedFiles: new Set([entryFile]) });
			await flushCache(compiler);

			write(unrelatedFile, 'globalThis.unrelated = "first edit";\n');
			await run(compiler, { modifiedFiles: new Set([unrelatedFile]) });
			await flushCache(compiler);

			write(unrelatedFile, 'globalThis.unrelated = "second edit";\n');
			await run(compiler, { modifiedFiles: new Set([unrelatedFile]) });
			await flushCache(compiler);
		} finally {
			await compiler.rspack.experiments.globalTrace.cleanup();
		}

		const trace = fs.readFileSync(traceFile, "utf8");
		savedModuleCounts = trace
			.split("\n")
			.map(line => line.match(/"message":"save (\d+) modules"/)?.[1])
			.filter(Boolean)
			.map(Number);
	},
	check() {
		expect(issuerSnapshots).toHaveLength(4);
		expect(issuerSnapshots[1]).not.toBe(issuerSnapshots[0]);
		expect(issuerSnapshots.slice(1)).toEqual([
			issuerSnapshots[1],
			issuerSnapshots[1],
			issuerSnapshots[1]
		]);
		expect(savedModuleCounts).toEqual([5, 2, 2, 2]);
	}
};
