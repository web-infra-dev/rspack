import type { OutputFileSystem } from "@rspack/core";
import { getSimpleProcessorRunner } from "../test/simple";
import type {
	ECompilerType,
	ITestContext,
	ITestEnv,
	TCompilation,
	TCompiler,
	TCompilerOptions,
	TCompilerStats,
	TCompilerStatsCompilation
} from "../type";
import { getCompiler } from "./common";

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
			await runner(name, {
				config: async (context: ITestContext) => {
					const compiler = getCompiler(context, name);
					const options = caseConfig.options?.(context) || {};
					options.mode ??= "production";
					options.context ??= context.getSource();
					options.entry ??= "./a.js";
					options.output ??= {};
					options.output.path ??= "/";
					options.output.pathinfo ??= true;
					options.optimization ??= {};
					options.optimization.minimize ??= false;
					compiler.setOptions(options);
				},
				compiler: async (context: ITestContext) => {
					const compiler = getCompiler(context, name);
					if (caseConfig.compilerCallback) {
						compiler.createCompilerWithCallback(caseConfig.compilerCallback);
					} else {
						compiler.createCompiler();
					}
					const c = compiler.getCompiler()!;
					c.outputFileSystem = {
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
					c.hooks.compilation.tap(
						"CompilerTest",
						compilation => ((compilation as any).bail = true)
					);
					await caseConfig.compiler?.(context, c);
				},
				build: async (context: ITestContext) => {
					const compiler = getCompiler(context, name);
					if (typeof caseConfig.build === "function") {
						await caseConfig.build?.(context, compiler.getCompiler()!);
					} else {
						await compiler.build();
					}
				},
				run: async (env: ITestEnv, context: ITestContext) => {},
				check: async (env: ITestEnv, context: ITestContext) => {
					const compiler = getCompiler(context, name);
					const c = compiler.getCompiler()!;
					const stats =
						compiler.getStats() as TCompilerStats<ECompilerType.Rspack>;
					if (caseConfig.error) {
						const statsJson = stats?.toJson({
							modules: true,
							reasons: true
						});
						const compilation = stats?.compilation;
						await caseConfig.check?.({
							context,
							compiler: c,
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
							compiler: c,
							compilation
						});
					} else {
						await caseConfig.check?.({
							context,
							files,
							compiler: c
						});
					}
				}
			});
		});
	}
}

export type TCompilerCaseConfig = {
	description: string;
	error?: boolean;
	skip?: boolean;
	options?: (context: ITestContext) => TCompilerOptions<ECompilerType.Rspack>;
	compiler?: (
		context: ITestContext,
		compiler: TCompiler<ECompilerType.Rspack>
	) => Promise<void>;
	build?: (
		context: ITestContext,
		compiler: TCompiler<ECompilerType.Rspack>
	) => Promise<void>;
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
	compilerCallback?: (
		error: Error | null,
		stats: TCompilerStats<ECompilerType.Rspack> | null
	) => void;
};
