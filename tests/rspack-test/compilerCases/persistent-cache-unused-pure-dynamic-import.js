const fs = require("node:fs");
const path = require("node:path");

const CASE_DIR = "persistent-cache-unused-pure-dynamic-import";
const CACHE_DIR = ".cache";
const OUTPUT_DIR = "output";
const WORK_DIR = "workdir";

async function recreateCompiler(context) {
	const compilerManager = context.getCompiler();
	await compilerManager.close();
	const compiler = compilerManager.createCompiler();
	compiler.outputFileSystem = fs;
}

function expectUnusedChunkNotEmitted(context) {
	expect(
		fs.existsSync(context.getDist(path.join(OUTPUT_DIR, "unused.js")))
	).toBe(false);
}

/** @type {import('@rspack/test-tools').TCompilerCaseConfig} */
module.exports = {
	description:
		"should recover used-by-exports for dynamic imports from persistent cache",
	options(context) {
		const sourceDir = path.resolve(__dirname, "../fixtures", CASE_DIR);
		const workDir = context.getDist(WORK_DIR);
		fs.rmSync(workDir, { recursive: true, force: true });
		fs.cpSync(sourceDir, workDir, { recursive: true });

		return {
			mode: "production",
			target: "node",
			context: workDir,
			entry: "./index.js",
			plugins: [
				new (require("@rspack/core").DefinePlugin)({
					FEATURE_ENABLED: "false"
				})
			],
			experiments: {
				cache: {
					type: "persistent",
					buildDependencies: [__filename],
					storage: {
						type: "filesystem",
						directory: context.getDist(CACHE_DIR)
					}
				}
			},
			optimization: {
				concatenateModules: false
			},
			output: {
				path: context.getDist(OUTPUT_DIR),
				filename: "main.js",
				chunkFilename: "[name].js",
				clean: true
			}
		};
	},
	async compiler(_, compiler) {
		compiler.outputFileSystem = fs;
	},
	async build(context) {
		const compilerManager = context.getCompiler();
		await compilerManager.build();
		expectUnusedChunkNotEmitted(context);

		await recreateCompiler(context);
		await compilerManager.build();
	},
	async check({ context }) {
		expectUnusedChunkNotEmitted(context);
	}
};
