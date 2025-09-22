import path from "node:path";
import type { Compiler, Stats } from "@rspack/core";
import fs from "fs-extra";
import { normalizePlaceholder } from "../helper/expect/placeholder";
import captureStdio from "../helper/legacy/captureStdio";
import { MultiTaskProcessor } from "../processor";
import { BasicCaseCreator } from "../test/creator";
import {
	ECompilerType,
	type ITestContext,
	type ITestEnv,
	type TCompiler,
	type TCompilerMultiStats,
	type TCompilerOptions,
	type TCompilerStats
} from "../type";

const REG_ERROR_CASE = /error$/;

function defaultOptions(
	index: number,
	context: ITestContext
): TCompilerOptions<ECompilerType.Rspack> {
	if (fs.existsSync(path.join(context.getSource(), "rspack.config.js"))) {
		return {
			experiments: {
				css: true,
				rspackFuture: {
					bundlerInfo: {
						force: false
					}
				}
			}
		} as TCompilerOptions<ECompilerType.Rspack>;
	}
	return {
		context: context.getSource(),
		mode: "development",
		entry: "./index.js",
		output: {
			filename: "bundle.js",
			path: context.getDist()
		},
		optimization: {
			minimize: false
		},
		experiments: {
			css: true,
			rspackFuture: {
				bundlerInfo: {
					force: false
				}
			},
			inlineConst: true,
			lazyBarrel: true
		}
	} as TCompilerOptions<ECompilerType.Rspack>;
}

function overrideOptions(
	index: number,
	context: ITestContext,
	options: TCompilerOptions<ECompilerType.Rspack>
) {
	if (!options.context) options.context = context.getSource();
	if (!options.output) options.output = options.output || {};
	if (!options.output.path) options.output.path = context.getDist();
	if (!options.plugins) options.plugins = [];
	if (!options.optimization) options.optimization = {};
	if (options.optimization.minimize === undefined) {
		options.optimization.minimize = false;
	}
	if (!global.printLogger) {
		options.infrastructureLogging = {
			level: "error"
		};
	}
}

class RspackStats {
	constructor(public value: string) {}
}

function createStatsProcessor(name: string) {
	const writeStatsOuptut = false;
	const snapshotName = "stats.txt";
	const processor = new MultiTaskProcessor<ECompilerType.Rspack>({
		name,
		compilerType: ECompilerType.Rspack,
		configFiles: ["rspack.config.js", "webpack.config.js"],
		runable: false,
		defaultOptions: (index, context) => defaultOptions(index, context),
		overrideOptions: (index, context, options) =>
			overrideOptions(index, context, options),
		async check(
			env: ITestEnv,
			context: ITestContext,
			compiler: TCompiler<ECompilerType.Rspack>,
			stats:
				| TCompilerStats<ECompilerType.Rspack>
				| TCompilerMultiStats<ECompilerType.Rspack>
				| null
		) {
			if (!stats || !compiler) return;

			for (const compilation of []
				.concat((stats as any).stats || stats)
				.map((s: any) => s.compilation)) {
				compilation.logging.delete("webpack.Compilation.ModuleProfile");
			}

			if (REG_ERROR_CASE.test(name)) {
				env.expect(stats.hasErrors()).toBe(true);
			} else if (stats.hasErrors()) {
				throw new Error(
					stats.toString({
						all: false,
						errors: true
						// errorStack: true,
						// errorDetails: true
					})
				);
			} else if (writeStatsOuptut) {
				fs.writeFileSync(
					path.join(context.getDist(), "stats.txt"),
					stats.toString({
						preset: "verbose",
						// context: context.getSource(),
						colors: false
					}),
					"utf-8"
				);
			}
			let toStringOptions: any = {
				context: context.getSource(),
				colors: false
			};
			let hasColorSetting = false;
			if (typeof compiler.options.stats !== "undefined") {
				toStringOptions = compiler.options.stats;
				if (toStringOptions === null || typeof toStringOptions !== "object")
					toStringOptions = { preset: toStringOptions };
				if (!toStringOptions.context)
					toStringOptions.context = context.getSource();
				hasColorSetting = typeof toStringOptions.colors !== "undefined";
			}

			if (Array.isArray(compiler.options) && !toStringOptions.children) {
				toStringOptions.children = compiler.options.map(o => o.stats);
			}

			// mock timestamps
			for (const { compilation: s } of [].concat(
				(stats as any).stats || stats
			) as Stats[]) {
				env.expect(s.startTime).toBeGreaterThan(0);
				env.expect(s.endTime).toBeGreaterThan(0);
				s.endTime = new Date("04/20/1970, 12:42:42 PM").getTime();
				s.startTime = s.endTime - 1234;
			}

			let actual = stats.toString(toStringOptions);
			env.expect(typeof actual).toBe("string");
			actual = stderr.toString() + actual;
			if (!hasColorSetting) {
				actual = actual
					.replace(/\u001b\[[0-9;]*m/g, "")
					// CHANGE: The time unit display in Rspack is second
					.replace(/[.0-9]+(\s?s)/g, "X$1")
					// CHANGE: Replace bundle size, since bundle sizes may differ between platforms
					.replace(/[0-9]+\.?[0-9]+ KiB/g, "xx KiB");
			}

			const snapshotPath = path.isAbsolute(snapshotName)
				? snapshotName
				: path.resolve(context.getSource(), `./__snapshots__/${snapshotName}`);

			env.expect(new RspackStats(actual)).toMatchFileSnapshot(snapshotPath);

			const testConfig = context.getTestConfig();
			if (typeof testConfig?.validate === "function") {
				testConfig.validate(stats, stderr.toString());
			}
		},

		async compiler(
			context: ITestContext,
			compiler: TCompiler<ECompilerType.Rspack>
		) {
			const compilers: Compiler[] = (compiler as any).compilers
				? (compiler as any).compilers
				: [compiler as any];
			for (const compiler of compilers) {
				if (!compiler.inputFileSystem) {
					continue;
				}
				const ifs = compiler.inputFileSystem;
				const inputFileSystem = Object.create(ifs);
				compiler.inputFileSystem = inputFileSystem;
				inputFileSystem.readFile = (...args: any[]) => {
					const callback = args.pop();
					ifs.readFile.apply(
						ifs,
						args.concat([
							(err: Error, result: Buffer) => {
								if (err) return callback(err);
								if (!/\.(js|json|txt)$/.test(args[0]))
									return callback(null, result);
								callback(null, normalizePlaceholder(result.toString("utf-8")));
							}
						]) as Parameters<typeof ifs.readFile>
					);
				};

				// CHANGE: The checkConstraints() function is currently not implemented in rspack
				// compiler.hooks.compilation.tap("StatsTestCasesTest", compilation => {
				// 	[
				// 		"optimize",
				// 		"optimizeModules",
				// 		"optimizeChunks",
				// 		"afterOptimizeTree",
				// 		"afterOptimizeAssets",
				// 		"beforeHash"
				// 	].forEach(hook => {
				// 		compilation.hooks[hook].tap("TestCasesTest", () =>
				// 			compilation.checkConstraints()
				// 		);
				// 	});
				// });
			}
		}
	});

	let stderr: any;
	processor.before = async (context: ITestContext) => {
		stderr = captureStdio(process.stderr, true);
	};
	processor.after = async (context: ITestContext) => {
		stderr.restore();
	};

	return processor;
}

const creator = new BasicCaseCreator({
	clean: true,
	describe: false,
	steps: ({ name }) => [createStatsProcessor(name)],
	description: () => "should print correct stats for"
});

export function createStatsOutputCase(name: string, src: string, dist: string) {
	creator.create(name, src, dist);
}
