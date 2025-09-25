import path from "node:path";
import type { OutputFileSystem } from "@rspack/core";
import { BasicCaseCreator } from "../test/creator";
import type {
	ECompilerType,
	ITestContext,
	ITestEnv,
	ITestProcessor,
	TCompilation,
	TCompiler,
	TCompilerOptions,
	TCompilerStats,
	TCompilerStatsCompilation
} from "../type";
import { getCompiler } from "./common";

function createCompilerProcessor(
	name: string,
	caseConfig: TCompilerCaseConfig
) {
	const logs = {
		mkdir: [] as string[],
		writeFile: [] as (string | number | Buffer<ArrayBufferLike>)[]
	};
	const files = {} as Record<string, string>;
	return {
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
			const stats = compiler.getStats() as TCompilerStats<ECompilerType.Rspack>;
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
	} as ITestProcessor;
}

const creator = new BasicCaseCreator({
	clean: true,
	steps: ({ name, caseConfig }) => {
		return [createCompilerProcessor(name, caseConfig as TCompilerCaseConfig)];
	},
	concurrent: false
});

const srcDir = path.resolve(
	__dirname,
	"../../../../tests/rspack-test/fixtures"
);
const distDir = path.resolve(
	__dirname,
	"../../../../tests/rspack-test/js/compiler"
);
export function defineCompileCase(
	name: string,
	caseConfigList: TCompilerCaseConfig | TCompilerCaseConfig[]
) {
	if (!Array.isArray(caseConfigList)) {
		caseConfigList = [caseConfigList];
	}
	for (let i = 0; i < caseConfigList.length; i++) {
		const caseConfig = caseConfigList[i];
		if (caseConfig.skip) {
			it.skip(`${name}[${i}]`, () => {});
			continue;
		}
		creator.create(
			`${name}[${i}]`,
			srcDir,
			path.join(distDir, name),
			undefined,
			{
				caseConfig,
				description: () => caseConfig.description
			}
		);
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
