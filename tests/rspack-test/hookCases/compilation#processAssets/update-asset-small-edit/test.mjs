const LARGE_BODY = "y".repeat(300 * 1024);
const INSERTED = "// inserted-by-plugin\n";
const SECOND = "/* second */\n";

const observed = [];

/** @type {import("@rspack/test-tools").THookCaseConfig} */
export default {
	description:
		"should keep exact bytes when updateAsset applies small insertions",
	options() {
		return {
			plugins: [
				function plugin(compiler) {
					const { Compilation, sources } = compiler.rspack;
					compiler.hooks.compilation.tap("test", compilation => {
						compilation.hooks.processAssets.tap(
							{
								name: "test",
								stage: Compilation.PROCESS_ASSETS_STAGE_ADDITIONAL
							},
							() => {
								compilation.emitAsset(
									"large.txt",
									new sources.RawSource(LARGE_BODY)
								);
							}
						);
						compilation.hooks.processAssets.tap(
							{
								name: "test",
								stage: Compilation.PROCESS_ASSETS_STAGE_OPTIMIZE
							},
							() => {
								// Two pure insertions in a row must keep the bytes intact.
								compilation.updateAsset(
									"large.txt",
									new sources.RawSource(INSERTED + LARGE_BODY)
								);
								compilation.updateAsset(
									"large.txt",
									new sources.RawSource(SECOND + INSERTED + LARGE_BODY)
								);
								// A middle replacement is not insert-only and must fall
								// back to the normal replacement path with identical bytes.
								const replaced =
									SECOND +
									INSERTED +
									LARGE_BODY.slice(0, 10) +
									"Z" +
									LARGE_BODY.slice(11);
								compilation.updateAsset(
									"large.txt",
									new sources.RawSource(replaced)
								);
								observed.push(
									compilation.getAsset("large.txt").source.source()
								);
							}
						);
						compilation.hooks.processAssets.tap(
							{
								name: "test",
								stage: Compilation.PROCESS_ASSETS_STAGE_REPORT
							},
							() => {
								const asset = compilation.getAsset("large.txt");
								observed.push(asset.source.source());
								observed.push(asset.source.buffer().toString());
								observed.push(asset.source.size());
							}
						);
					});
				}
			]
		};
	},
	async check() {
		const expected =
			SECOND +
			INSERTED +
			LARGE_BODY.slice(0, 10) +
			"Z" +
			LARGE_BODY.slice(11);
		expect(observed[0]).toBe(expected);
		expect(observed[1]).toBe(expected);
		expect(observed[2]).toBe(expected);
		expect(observed[3]).toBe(expected.length);
	}
};
