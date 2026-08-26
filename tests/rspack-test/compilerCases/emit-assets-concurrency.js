const { createFsFromVolume, Volume } = require("memfs");
const { RawSource } = require("@rspack/core").sources;

const ASSET_COUNT = 200;
const CONCURRENCY_LIMIT = 15;

let peakConcurrentWrites;
let totalWrites;

/** @type {import('@rspack/test-tools').TCompilerCaseConfig} */
module.exports = {
	description: "should bound the number of concurrent asset writes during emit",
	options(context) {
		return {
			entry: "./a",
			output: {
				path: context.getDist(),
				filename: "main.js"
			},
			plugins: [
				{
					apply(compiler) {
						compiler.hooks.thisCompilation.tap("EmitManyAssets", compilation => {
							compilation.hooks.processAssets.tap("EmitManyAssets", () => {
								for (let i = 0; i < ASSET_COUNT; i++) {
									compilation.emitAsset(`extra-${i}.txt`, new RawSource(`${i}`));
								}
							});
						});
					}
				}
			]
		};
	},
	compiler(context, compiler) {
		peakConcurrentWrites = 0;
		totalWrites = 0;

		const outputFileSystem = createFsFromVolume(new Volume());
		const writeFile = outputFileSystem.writeFile.bind(outputFileSystem);
		let concurrentWrites = 0;

		outputFileSystem.writeFile = (filename, content, callback) => {
			concurrentWrites++;
			totalWrites++;
			peakConcurrentWrites = Math.max(peakConcurrentWrites, concurrentWrites);

			writeFile(filename, content, error => {
				setImmediate(() => {
					concurrentWrites--;
					callback(error);
				});
			});
		};

		compiler.outputFileSystem = outputFileSystem;
	},
	check() {
		expect(totalWrites).toBe(ASSET_COUNT + 1);
		expect(peakConcurrentWrites).toBeGreaterThan(1);
		expect(peakConcurrentWrites).toBeLessThanOrEqual(CONCURRENCY_LIMIT);
	}
};
