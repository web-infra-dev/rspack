const LARGE_BODY = "y".repeat(300 * 1024);
const INSERTED = "// inserted-by-plugin\n";
const SECOND = "/* second */\n";

const expectedLargeMap =
	SECOND + INSERTED + LARGE_BODY.slice(0, 10) + "Z" + LARGE_BODY.slice(11);
const expectedCode = INSERTED + LARGE_BODY;
const expectedBinary = Buffer.concat([
	Buffer.from("// prefix\n"),
	Buffer.from(LARGE_BODY)
]);
const expectedMapped = INSERTED + LARGE_BODY;
const providedMap = {
	version: 3,
	file: "mapped.map",
	sources: ["a.js"],
	names: [],
	mappings: "AAAA"
};

const observed = {};

/** @type {import("@rspack/test-tools").THookCaseConfig} */
export default {
	description:
		"should keep exact bytes for small updateAsset insertions without changing source semantics",
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
									"large.map",
									new sources.RawSource(LARGE_BODY)
								);
								compilation.emitAsset(
									"code.txt",
									new sources.RawSource(LARGE_BODY)
								);
								compilation.emitAsset(
									"binary.map",
									new sources.RawSource(Buffer.from(LARGE_BODY))
								);
								compilation.emitAsset(
									"mapped.map",
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
								// Two pure insertions in a row on a serialized map keep
								// the previous source plus an edit; bytes stay intact.
								compilation.updateAsset(
									"large.map",
									new sources.RawSource(INSERTED + LARGE_BODY)
								);
								compilation.updateAsset(
									"large.map",
									new sources.RawSource(SECOND + INSERTED + LARGE_BODY)
								);
								// A middle replacement is not insert-only and falls back
								// to the normal replacement path.
								compilation.updateAsset(
									"large.map",
									new sources.RawSource(expectedLargeMap)
								);
								// Non-`.map` assets never take the optimization, so a
								// plugin-supplied source map cannot be dropped.
								compilation.updateAsset(
									"code.txt",
									new sources.RawSource(expectedCode)
								);
								// Buffer-valued updates keep their binary representation.
								compilation.updateAsset(
									"binary.map",
									new sources.RawSource(expectedBinary)
								);
								// A SourceMapSource keeps the map the plugin supplied.
								compilation.updateAsset(
									"mapped.map",
									new sources.SourceMapSource(
										expectedMapped,
										"mapped.map",
										providedMap
									)
								);
							}
						);
						compilation.hooks.processAssets.tap(
							{
								name: "test",
								stage: Compilation.PROCESS_ASSETS_STAGE_REPORT
							},
							() => {
								const asset = name => compilation.getAsset(name).source;
								observed.largeMap = asset("large.map").source();
								observed.largeMapBuffer = asset("large.map")
									.buffer()
									.toString();
								observed.largeMapSize = asset("large.map").size();
								observed.code = asset("code.txt").source();
								observed.binaryIsBuffer = Buffer.isBuffer(
									asset("binary.map").source()
								);
								observed.binary = asset("binary.map").buffer().toString();
								observed.mappedValue = asset("mapped.map").source();
								observed.mappedMap = asset("mapped.map").map();
							}
						);
					});
				}
			]
		};
	},
	async check() {
		expect(observed.largeMap).toBe(expectedLargeMap);
		expect(observed.largeMapBuffer).toBe(expectedLargeMap);
		expect(observed.largeMapSize).toBe(expectedLargeMap.length);
		expect(observed.code).toBe(expectedCode);
		expect(observed.binaryIsBuffer).toBe(true);
		expect(observed.binary).toBe(expectedBinary.toString());
		expect(observed.mappedValue).toBe(expectedMapped);
		expect(observed.mappedMap).toBeTruthy();
		expect(observed.mappedMap.mappings).toBe("AAAA");
	}
};
