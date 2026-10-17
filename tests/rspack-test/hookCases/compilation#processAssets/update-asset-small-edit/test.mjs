const LARGE_BODY = "y".repeat(300 * 1024);
const INSERTED = "// inserted-by-plugin\n";
const SECOND = "/* second */\n";

const expectedLargeMap =
	SECOND + INSERTED + LARGE_BODY.slice(0, 10) + "Z" + LARGE_BODY.slice(11);
const expectedCode = INSERTED + LARGE_BODY;
const expectedConcat = INSERTED + LARGE_BODY + "\n";
const expectedRawMapped = INSERTED + LARGE_BODY;
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
									"concat.txt",
									new sources.ConcatSource(
										new sources.RawSource(LARGE_BODY),
										new sources.RawSource("\n")
									)
								);
								compilation.emitAsset(
									"binary.map",
									new sources.RawSource(Buffer.from(LARGE_BODY))
								);
								compilation.emitAsset(
									"raw-replaces-mapped.js",
									new sources.SourceMapSource(
										LARGE_BODY,
										"raw-replaces-mapped.js",
										providedMap
									)
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
								// Any large plain-text asset can take the optimization.
								compilation.updateAsset(
									"code.txt",
									new sources.RawSource(expectedCode)
								);
								// Concatenated sources can take it too.
								compilation.updateAsset(
									"concat.txt",
									new sources.RawSource(expectedConcat)
								);
								// Buffer-valued updates keep their binary representation.
								compilation.updateAsset(
									"binary.map",
									new sources.RawSource(expectedBinary)
								);
								// Replacing a mapped source with a raw source
								// drops the map, matching the normal path.
								compilation.updateAsset(
									"raw-replaces-mapped.js",
									new sources.RawSource(expectedRawMapped)
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
								observed.concat = asset("concat.txt").source();
								observed.binaryIsBuffer = Buffer.isBuffer(
									asset("binary.map").source()
								);
								observed.binary = asset("binary.map").buffer().toString();
								observed.rawMappedValue = asset(
									"raw-replaces-mapped.js"
								).source();
								observed.rawMappedMap = asset("raw-replaces-mapped.js").map();
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
		expect(observed.concat).toBe(expectedConcat);
		expect(observed.binaryIsBuffer).toBe(true);
		expect(observed.binary).toBe(expectedBinary.toString());
		expect(observed.rawMappedValue).toBe(expectedRawMapped);
		expect(observed.rawMappedMap).toBeUndefined();
		expect(observed.mappedValue).toBe(expectedMapped);
		expect(observed.mappedMap).toBeTruthy();
		expect(observed.mappedMap.mappings).toBe("AAAA");
	}
};
