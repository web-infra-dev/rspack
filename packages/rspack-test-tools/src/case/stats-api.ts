import type fs from "node:fs";
import { createFsFromVolume, Volume } from "memfs";
import { SimpleTaskProcessor } from "../processor";
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

type TStatsAPICaseConfig = {
	description: string;
	options?: (context: ITestContext) => TCompilerOptions<ECompilerType.Rspack>;
	snapshotName?: string;
	compiler?: (
		context: ITestContext,
		compiler: TCompiler<ECompilerType.Rspack>
	) => Promise<void>;
	build?: (
		context: ITestContext,
		compiler: TCompiler<ECompilerType.Rspack>
	) => Promise<void>;
	check?: (
		stats: TCompilerStats<ECompilerType.Rspack>,
		compiler: TCompiler<ECompilerType.Rspack>
	) => Promise<void>;
};

function options(
	context: ITestContext,
	custom?: (context: ITestContext) => TCompilerOptions<ECompilerType.Rspack>
) {
	const res = (custom?.(context) ||
		{}) as TCompilerOptions<ECompilerType.Rspack>;
	res.experiments ??= {};
	res.experiments!.css ??= true;
	res.experiments!.rspackFuture ??= {};
	res.experiments!.rspackFuture!.bundlerInfo ??= {};
	res.experiments!.rspackFuture!.bundlerInfo!.force ??= false;
	if (!global.printLogger) {
		res.infrastructureLogging = {
			level: "error"
		};
	}
	return res;
}

async function compiler(
	context: ITestContext,
	compiler: TCompiler<ECompilerType.Rspack>,
	custom?: (
		context: ITestContext,
		compiler: TCompiler<ECompilerType.Rspack>
	) => Promise<void>
) {
	if (custom) {
		await custom(context, compiler);
	}
	if (compiler) {
		compiler.outputFileSystem = createFsFromVolume(
			new Volume()
		) as unknown as typeof fs;
	}
}

async function check(
	env: ITestEnv,
	context: ITestContext,
	compiler: TCompiler<ECompilerType.Rspack>,
	stats: TCompilerStats<ECompilerType.Rspack>,
	custom?: (
		stats: TCompilerStats<ECompilerType.Rspack>,
		compiler: TCompiler<ECompilerType.Rspack>
	) => Promise<void>
) {
	env.expect(typeof stats).toBe("object");
	await custom?.(stats, compiler);
}

export function createStatsAPICase(
	name: string,
	src: string,
	dist: string,
	testConfig: string
) {
	if (!addedSerializer) {
		addedSerializer = true;
	}
	const caseConfig: TStatsAPICaseConfig = require(testConfig);
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
