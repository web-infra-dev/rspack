function shuffle(arr) {
	let m = arr.length;
	while (m > 1) {
		let index = Math.floor(Math.random() * m--);
		[arr[m] , arr[index]] = [arr[index] , arr[m]]
	}
	return arr;
}

const NAME = "TestPlugin";

/**
 * @type {import('@rspack/core').RspackOptions}
 */
module.exports = {
	plugins: [
		{
			name: NAME,
			apply(compiler) {
				const { sources: { ConcatSource, RawSource }, Compilation } = compiler.webpack;
				compiler.hooks.compilation.tap("compilation", compilation => {
					function addStage(stage) {
						compilation.hooks.processAssets.tapPromise(
							{
								name: NAME,
								stage,
							},
							async assets => {
								for (const [key, value] of Object.entries(assets)) {
									compilation.updateAsset(
										key,
										new ConcatSource(new RawSource(`//${stage};\n`), value)
									);
								}
							}
						);
					}
					const stages = [
						Compilation.PROCESS_ASSETS_STAGE_ADDITIONAL,
						Compilation.PROCESS_ASSETS_STAGE_PRE_PROCESS,
						Compilation.PROCESS_ASSETS_STAGE_DERIVED,
						Compilation.PROCESS_ASSETS_STAGE_ADDITIONS,
						Compilation.PROCESS_ASSETS_STAGE_NONE,
						Compilation.PROCESS_ASSETS_STAGE_OPTIMIZE,
						Compilation.PROCESS_ASSETS_STAGE_OPTIMIZE_COUNT,
						Compilation.PROCESS_ASSETS_STAGE_OPTIMIZE_COMPATIBILITY,
						Compilation.PROCESS_ASSETS_STAGE_OPTIMIZE_SIZE,
						Compilation.PROCESS_ASSETS_STAGE_DEV_TOOLING,
						Compilation.PROCESS_ASSETS_STAGE_OPTIMIZE_INLINE,
						Compilation.PROCESS_ASSETS_STAGE_SUMMARIZE,
						Compilation.PROCESS_ASSETS_STAGE_OPTIMIZE_HASH,
						Compilation.PROCESS_ASSETS_STAGE_OPTIMIZE_TRANSFER,
						Compilation.PROCESS_ASSETS_STAGE_ANALYSE,
						Compilation.PROCESS_ASSETS_STAGE_REPORT,
					].flatMap((s, i) => {
						const r = i + 1;
						return [s - r, s, s + r]
					})
					shuffle(stages);
					stages.forEach(s => addStage(s))
				});
			}
		}
	]
};
