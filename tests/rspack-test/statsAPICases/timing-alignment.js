const BUILD_DELAY_MS = 500;
const FINISH_DELAY_MS = 200;
const AFTER_SEAL_DELAY_MS = 100;

const sleep = ms => new Promise(resolve => setTimeout(resolve, ms));

class TimingBoundaryPlugin {
	apply(compiler) {
		compiler.hooks.compilation.tap("TimingBoundaryPlugin", compilation => {
			compilation.hooks.finishModules.tapPromise(
				"TimingBoundaryPlugin",
				async () => {
					await sleep(FINISH_DELAY_MS);
				}
			);
			compilation.hooks.afterSeal.tapPromise(
				"TimingBoundaryPlugin",
				async () => {
					await sleep(AFTER_SEAL_DELAY_MS);
				}
			);
		});
	}
}

const getTiming = (logging, origin, label) => {
	const entry = logging?.[origin]?.entries.find(
		entry => entry.type === "time" && entry.message.startsWith(`${label}: `)
	);
	expect(entry).toBeDefined();

	const match = /: ([\d.]+) ms$/.exec(entry.message);
	expect(match).not.toBeNull();

	return Number(match[1]);
};

/** @type {import('@rspack/test-tools').TStatsAPICaseConfig} */
module.exports = {
	description: "should align top-level timing boundaries with webpack",
	options(context) {
		return {
			context: context.getSource(),
			entry: "./fixtures/abc",
			mode: "development",
			module: {
				rules: [
					{
						test: /abc\.js$/,
						use: require.resolve("../fixtures/delay-loader")
					}
				]
			},
			plugins: [new TimingBoundaryPlugin()]
		};
	},
	async check(stats) {
		const logging = stats?.toJson({ all: false, logging: "verbose" }).logging;
		const makeHook = getTiming(logging, "rspack.Compiler", "make hook");
		const buildModuleGraph = getTiming(
			logging,
			"rspack.Compiler",
			"build module graph"
		);
		const finishCompilation = getTiming(
			logging,
			"rspack.Compiler",
			"finish compilation"
		);
		const finishModules = getTiming(
			logging,
			"rspack.Compilation",
			"finish modules"
		);
		const sealCompilation = getTiming(
			logging,
			"rspack.Compiler",
			"seal compilation"
		);
		const afterSeal = getTiming(logging, "rspack.Compilation", "after seal");

		expect(buildModuleGraph).toBeGreaterThanOrEqual(BUILD_DELAY_MS * 0.8);
		expect(finishModules).toBeGreaterThanOrEqual(FINISH_DELAY_MS * 0.8);
		expect(afterSeal).toBeGreaterThanOrEqual(AFTER_SEAL_DELAY_MS * 0.8);

		expect(makeHook).toBeGreaterThanOrEqual(buildModuleGraph * 0.9);
		expect(finishCompilation).toBeGreaterThanOrEqual(finishModules * 0.9);

		expect(makeHook).toBeGreaterThan(sealCompilation);
		expect(sealCompilation).toBeGreaterThanOrEqual(afterSeal * 0.9);
		expect(sealCompilation).toBeLessThan(buildModuleGraph * 0.9);
	}
};
