const fs = require("node:fs");
const path = require("node:path");
const vm = require("node:vm");

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

function run(compiler, modifiedFiles) {
	return new Promise((resolve, reject) => {
		compiler.run(
			(error, stats) => {
				if (error) return reject(error);
				if (stats.hasErrors()) {
					return reject(
						new Error(stats.toString({ all: false, errors: true }))
					);
				}
				resolve();
			},
			{ modifiedFiles: new Set(modifiedFiles) }
		);
	});
}

const cases = [false, true].flatMap((cache) =>
	[makeOnlyIncremental, true].map((incremental) => ({
		cache,
		incremental,
		shared: false
	}))
);
cases.push(
	...[makeOnlyIncremental, true].map((incremental) => ({
		cache: true,
		incremental,
		shared: true
	}))
);

module.exports = cases.map(({ cache, incremental, shared }) => {
	let root;
	let entryFile;
	let sideFile;
	let builtModules;

	/** @type {import('@rspack/test-tools').TCompilerCaseConfig} */
	return {
		description: `should restore side-effect imports after metadata changes (cache=${cache}, incremental=${incremental === true ? "all" : "make"}, shared=${shared})`,
		options(context) {
			root = context.getDist("side-effects-metadata-rebuild");
			entryFile = path.join(root, "index.js");
			sideFile = path.join(root, "pkg/side.js");
			fs.mkdirSync(path.join(root, "pkg"), { recursive: true });
			fs.writeFileSync(
				entryFile,
				'import "./pkg/side.js";\nimport "./unrelated.js";\n'
			);
			fs.writeFileSync(
				sideFile,
				"globalThis.sideEffect = true;\nexport const value = 42;\n"
			);
			if (shared) {
				fs.writeFileSync(
					path.join(root, "retained.js"),
					'import { value } from "./pkg/side.js";\nglobalThis.value = value;\n'
				);
			}
			fs.writeFileSync(
				path.join(root, "unrelated.js"),
				"globalThis.unrelated = 1;\n"
			);

			return {
				context: root,
				mode: "development",
				devtool: false,
				entry: shared
					? { main: "./index.js", retained: "./retained.js" }
					: "./index.js",
				cache,
				incremental,
				optimization: {
					sideEffects: true,
					usedExports: true,
					concatenateModules: false,
					minimize: false
				},
				output: { path: path.join(root, "dist"), filename: "[name].js" },
				plugins: [
					{
						apply(compiler) {
							compiler.hooks.compilation.tap(
								"TrackModuleBuilds",
								(compilation) => {
									builtModules = new Set();
									compilation.hooks.buildModule.tap(
										"TrackModuleBuilds",
										(module) => {
											builtModules.add(module.resource);
										}
									);
								}
							);
						}
					}
				]
			};
		},
		compiler(_context, compiler) {
			compiler.outputFileSystem = fs;
		},
		async build(_context, compiler) {
			const packageFile = path.join(root, "pkg/package.json");
			const bundleFile = path.join(root, "dist/main.js");
			const sideEffects = [true, false, true, false, true];
			for (const [step, enabled] of sideEffects.entries()) {
				fs.writeFileSync(packageFile, JSON.stringify({ sideEffects: enabled }));
				await run(compiler, step === 0 ? [] : [packageFile]);

				const globals = {};
				vm.runInNewContext(fs.readFileSync(bundleFile, "utf8"), globals);
				expect(globals.sideEffect).toBe(enabled ? true : undefined);
				expect(globals.unrelated).toBe(1);
				expect(builtModules.has(entryFile)).toBe(step === 0);
				if (!enabled && !shared) expect(builtModules.has(sideFile)).toBe(false);
				if (shared) {
					const retained = {};
					vm.runInNewContext(
						fs.readFileSync(path.join(root, "dist/retained.js"), "utf8"),
						retained
					);
					expect(retained.value).toBe(42);
				}
			}

			const unrelatedFile = path.join(root, "unrelated.js");
			fs.writeFileSync(unrelatedFile, "globalThis.unrelated = 2;\n");
			await run(compiler, [unrelatedFile]);
			const globals = {};
			vm.runInNewContext(fs.readFileSync(bundleFile, "utf8"), globals);
			expect(globals.sideEffect).toBe(true);
			expect(globals.unrelated).toBe(2);
			expect(builtModules.has(entryFile)).toBe(false);
			expect(builtModules.has(sideFile)).toBe(false);
		}
	};
});
