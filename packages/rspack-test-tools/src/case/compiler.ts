import type { OutputFileSystem } from "@rspack/core";
import {
	type ISimpleProcessorOptions,
	SimpleTaskProcessor
} from "../processor";
import { getSimpleProcessorRunner } from "../test/simple";
import {
	ECompilerType,
	type ITestContext,
	type TCompilation,
	type TCompiler,
	type TCompilerStatsCompilation
} from "../type";

export type TCompilerCaseConfig = Omit<
	ISimpleProcessorOptions<ECompilerType.Rspack>,
	"name" | "compilerType" | "check"
> & {
	description: string;
	error?: boolean;
	skip?: boolean;
	check?: ({
		context,
		stats,
		files,
		compiler,
		compilation
	}: {
		context: ITestContext;
		stats?: TCompilerStatsCompilation<ECompilerType.Rspack>;
		files?: Record<string, string>;
		compiler: TCompiler<ECompilerType.Rspack>;
		compilation?: TCompilation<ECompilerType.Rspack>;
	}) => Promise<void>;
};

export function createCompilerCase(
	name: string,
	src: string,
	dist: string,
	testConfig: string
) {
	let caseConfigList: TCompilerCaseConfig | TCompilerCaseConfig[] = require(
		testConfig
	);
	if (!Array.isArray(caseConfigList)) {
		caseConfigList = [caseConfigList];
	}
	const runner = getSimpleProcessorRunner(src, dist);

	for (const caseConfig of caseConfigList) {
		const testFn = caseConfig.skip ? it.skip : it;
		testFn(caseConfig.description, async () => {
			const logs = {
				mkdir: [] as string[],
				writeFile: [] as (string | number | Buffer<ArrayBufferLike>)[]
			};
			const files = {} as Record<string, string>;
			await runner(
				name,
				new SimpleTaskProcessor({
					name: name,
					compilerType: ECompilerType.Rspack,
					compilerCallback: caseConfig.compilerCallback,
					build: caseConfig.build,

					options: context => {
						const options = caseConfig.options?.(context) || {};
						options.mode ??= "production";
						options.context ??= context.getSource();
						options.entry ??= "./a.js";
						options.output ??= {};
						options.output.path ??= "/";
						options.output.pathinfo ??= true;
						options.optimization ??= {};
						options.optimization.minimize ??= false;
						return options;
					},
					async compiler(context, compiler) {
						compiler.outputFileSystem = {
							// CHANGE: Added support for the `options` parameter to enable recursive directory creation,
							// accommodating Rspack's requirement that differs from webpack's usage
							mkdir(
								path: string,
								callback: (
									err?: Error & {
										code?: string;
									}
								) => void
							) {
								const recursive = false;
								// if (typeof options === "function") {
								// 	callback = options;
								// } else if (options) {
								// 	if (options.recursive !== undefined) recursive = options.recursive;
								// }
								logs.mkdir.push(path);
								if (recursive) {
									callback();
								} else {
									const err = new Error() as Error & {
										code?: string;
									};
									err.code = "EEXIST";
									callback(err);
								}
							},
							writeFile(name, content, callback) {
								logs.writeFile.push(name, content);
								files[name] = content.toString("utf-8");
								callback();
							},
							stat(path, callback) {
								callback(new Error("ENOENT"));
							}
						} as OutputFileSystem;
						compiler.hooks.compilation.tap(
							"CompilerTest",
							compilation => ((compilation as any).bail = true)
						);
						await caseConfig.compiler?.(context, compiler);
					},
					async check(env, context, compiler, stats) {
						if (caseConfig.error) {
							const statsJson = stats?.toJson({
								modules: true,
								reasons: true
							});
							const compilation = stats?.compilation;
							await caseConfig.check?.({
								context,
								compiler,
								stats: statsJson,
								compilation,
								files
							});
						} else if (stats) {
							expect(typeof stats).toBe("object");
							const compilation = stats.compilation;
							const statsJson = stats.toJson({
								modules: true,
								reasons: true
							});
							expect(typeof statsJson).toBe("object");
							expect(statsJson).toHaveProperty("errors");
							expect(Array.isArray(statsJson.errors)).toBe(true);
							if (statsJson.errors!.length > 0) {
								expect(statsJson.errors![0]).toBeInstanceOf(Object);
								throw statsJson.errors![0];
							}
							statsJson.logs = logs;
							await caseConfig.check?.({
								context,
								stats: statsJson,
								files,
								compiler,
								compilation
							});
						} else {
							await caseConfig.check?.({
								context,
								files,
								compiler
							});
						}
					}
				})
			);
		});
	}
}
