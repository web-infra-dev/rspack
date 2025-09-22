import type fs from "node:fs";
import path from "node:path";
import type { StatsError } from "@rspack/core";
import merge from "webpack-merge";
import {
	type ISimpleProcessorOptions,
	SimpleTaskProcessor
} from "../processor";
import { getSimpleProcessorRunner } from "../test/simple";
import {
	ECompilerType,
	type ITestContext,
	type ITestEnv,
	type TCompiler,
	type TCompilerOptions,
	type TCompilerStats
} from "../type";

let addedSerializer = false;

function options<T extends ECompilerType.Rspack>(
	context: ITestContext,
	custom: (
		context: ITestContext,
		options: TCompilerOptions<T>
	) => TCompilerOptions<T>
): TCompilerOptions<T> {
	let options = {
		context: path.resolve(
			__dirname,
			"../../../../tests/rspack-test/fixtures/errors"
		),
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
	_: TCompiler<ECompilerType.Rspack>,
	stats: TCompilerStats<ECompilerType.Rspack>,
	check?: (stats: RspackStatsDiagnostics) => Promise<void>
) {
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

export type TErrorCaseConfig = Omit<
	ISimpleProcessorOptions<ECompilerType.Rspack>,
	"name" | "compilerType"
> & {
	description: string;
};

export function createErrorCase(
	name: string,
	src: string,
	dist: string,
	testConfig: string
) {
	if (!addedSerializer) {
		addedSerializer = true;
	}
	const caseConfig = require(testConfig);
	const runner = getSimpleProcessorRunner(src, dist);

	it(caseConfig.description, async () => {
		await runner(
			name,
			new SimpleTaskProcessor({
				name: name,
				compilerType: ECompilerType.Rspack,
				options: context => options(context, caseConfig.options),
				compiler: (context, c) => compiler(context, c, caseConfig.compiler),
				build: caseConfig.build,
				check: (env, context, compiler, stats) =>
					check(env, context, compiler, stats, caseConfig.check)
			})
		);
	});
}
