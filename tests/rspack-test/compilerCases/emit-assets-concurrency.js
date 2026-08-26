const { createFsFromVolume, Volume } = require("memfs");

const ADDITIONAL_ASSET_COUNT = 30;
const MAX_CONCURRENT_ASSET_EMITS = 15;

let activeWrites;
let buildError;
let buildStats;
let maxActiveWrites;
let outputFileSystem;
let writeCount;

/** @type {import('@rspack/test-tools').TCompilerCaseConfig} */
module.exports = {
	description: "should limit concurrent asset emits",
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
						compiler.hooks.compilation.tap("EmitAssetsPlugin", compilation => {
							compilation.hooks.processAssets.tap("EmitAssetsPlugin", () => {
								const { RawSource } = require("webpack-sources");
								for (let i = 0; i < ADDITIONAL_ASSET_COUNT; i++) {
									compilation.emitAsset(
										`asset-${i}.txt`,
										new RawSource(`asset ${i}`)
									);
								}
							});
						});
					}
				}
			]
		};
	},
	compiler(context, compiler) {
		activeWrites = 0;
		buildError = undefined;
		buildStats = undefined;
		maxActiveWrites = 0;
		writeCount = 0;
		outputFileSystem = createFsFromVolume(new Volume());
		const writeFile = outputFileSystem.writeFile.bind(outputFileSystem);

		outputFileSystem.writeFile = (filename, content, callback) => {
			activeWrites++;
			writeCount++;
			maxActiveWrites = Math.max(maxActiveWrites, activeWrites);

			writeFile(filename, content, error => {
				setTimeout(() => {
					activeWrites--;
					callback(error);
				}, 10);
			});
		};

		compiler.outputFileSystem = outputFileSystem;
	},
	build(context, compiler) {
		return new Promise(resolve => {
			compiler.run((error, stats) => {
				buildError = error;
				buildStats = stats;
				resolve();
			});
		});
	},
	check() {
		expect(buildError).toBeFalsy();
		expect(buildStats).toBeTruthy();
		expect(activeWrites).toBe(0);
		expect(writeCount).toBe(ADDITIONAL_ASSET_COUNT + 1);
		expect(maxActiveWrites).toBe(MAX_CONCURRENT_ASSET_EMITS);
	}
};
