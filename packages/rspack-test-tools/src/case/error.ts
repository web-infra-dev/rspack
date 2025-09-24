import type fs from "node:fs";
import path from "node:path";
import type { StatsError } from "@rspack/core";
import merge from "webpack-merge";
import { BasicCaseCreator } from "../test/creator";
import type {
	ECompilerType,
	ITestContext,
	ITestEnv,
	TCompiler,
	TCompilerOptions
} from "../type";
import { getCompiler } from "./common";

let addedSerializer = false;

const creator = new BasicCaseCreator({
	clean: true,
	describe: true,
	steps: ({ name, caseConfig }) => {
		const config = caseConfig as TErrorCaseConfig;
		return [
			{
				config: async (context: ITestContext) => {
					const compiler = getCompiler(context, name);
					compiler.setOptions(options(context, config.options));
				},
				compiler: async (context: ITestContext) => {
					const compilerManager = getCompiler(context, name);
					compilerManager.createCompiler();
					compiler(context, compilerManager.getCompiler()!, config.compiler);
				},
				build: async (context: ITestContext) => {
					const compiler = getCompiler(context, name);
					if (typeof config.build === "function") {
						await config.build(context, compiler.getCompiler()!);
					} else {
						await compiler.build();
					}
				},
				run: async (env: ITestEnv, context: ITestContext) => {
					// no need to run, just check the snapshot of diagnostics
				},
				check: async (env: ITestEnv, context: ITestContext) => {
					await check(env, context, name, config.check);
				}
			}
		];
	},
	concurrent: true
});

const srcDir = path.resolve(
	__dirname,
	"../../../../tests/rspack-test/fixtures"
);
const distDir = path.resolve(
	__dirname,
	"../../../../tests/rspack-test/js/error"
);
export function defineErrorCase(name: string, caseConfig: TErrorCaseConfig) {
	if (!addedSerializer) {
		addedSerializer = true;
	}
	creator.create(
		name,
		path.resolve(srcDir, "errors"),
		path.join(distDir, name),
		undefined,
		{
			caseConfig,
			description: () => caseConfig.description
		}
	);
}

function options<T extends ECompilerType.Rspack>(
	context: ITestContext,
	custom?: (
		context: ITestContext,
		options: TCompilerOptions<T>
	) => TCompilerOptions<T>
): TCompilerOptions<T> {
	let options = {
		context: context.getSource(),
		mode: "none",
		devtool: false,
		optimization: {
			minimize: false,
			moduleIds: "named",
			chunkIds: "named"
		},
		experiments: {
			css: true,
			rspackFuture: {
				bundlerInfo: {
					force: false
				}
			}
		}
	} as TCompilerOptions<T>;
	if (typeof custom === "function") {
		options = merge(options, custom(context, options));
	}
	if (options.mode === "production") {
		if (options.optimization) options.optimization.minimize = true;
		else options.optimization = { minimize: true };
	}
	return options;
}

async function compiler<T extends ECompilerType.Rspack>(
	context: ITestContext,
	compiler: TCompiler<T>,
	custom?: (context: ITestContext, compiler: TCompiler<T>) => Promise<void>
) {
	if (compiler) {
		compiler.outputFileSystem = {
			// CHANGE: rspack outputFileSystem `mkdirp` uses option `{ recursive: true }`, webpack's second parameter is alway a callback
			mkdir(
				dir: string,
				maybeOptionOrCallback: unknown,
				maybeCallback: unknown
			) {
				if (typeof maybeOptionOrCallback === "function") {
					maybeOptionOrCallback();
				} else if (typeof maybeCallback === "function") {
					maybeCallback();
				}
			},
			writeFile(file: string, content: string, callback: () => {}) {
				callback();
			},
			stat(file: string, callback: (e: Error) => {}) {
				callback(new Error("ENOENT"));
			},
			mkdirSync() {},
			writeFileSync() {}
		} as unknown as typeof fs;
	}
	await custom?.(context, compiler);
}

class RspackStatsDiagnostics {
	constructor(
		public errors: StatsError[],
		public warnings: StatsError[]
	) {}
}

async function check(
	env: ITestEnv,
	context: ITestContext,
	name: string,
	check?: (stats: RspackStatsDiagnostics) => Promise<void>
) {
	const compiler = getCompiler(context, name);
	const stats = compiler.getStats();
	env.expect(typeof stats).toBe("object");
	const statsResult = stats!.toJson({ errorDetails: false });
	env.expect(typeof statsResult).toBe("object");
	const { errors, warnings } = statsResult;
	env.expect(Array.isArray(errors)).toBe(true);
	env.expect(Array.isArray(warnings)).toBe(true);

	await check?.(
		new RspackStatsDiagnostics(errors as StatsError[], warnings as StatsError[])
	);
}

export type TErrorCaseConfig = {
	description: string;
	options?: (context: ITestContext) => TCompilerOptions<ECompilerType.Rspack>;
	compiler?: (
		context: ITestContext,
		compiler: TCompiler<ECompilerType.Rspack>
	) => Promise<void>;
	build?: (
		context: ITestContext,
		compiler: TCompiler<ECompilerType.Rspack>
	) => Promise<void>;
	check?: (stats: RspackStatsDiagnostics) => Promise<void>;
};
