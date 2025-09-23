import type fs from "node:fs";
import { createFsFromVolume, Volume } from "memfs";
import { getSimpleProcessorRunner } from "../test/simple";
import type {
	ECompilerType,
	ITestContext,
	ITestEnv,
	TCompiler,
	TCompilerOptions,
	TCompilerStats
} from "../type";
import { getCompiler } from "./common";

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
		await runner(name, {
			config: async (context: ITestContext) => {
				const compiler = getCompiler(context, name);
				compiler.setOptions(options(context, caseConfig.options));
			},
			compiler: async (context: ITestContext) => {
				const compilerManager = getCompiler(context, name);
				compilerManager.createCompiler();
				compiler(context, compilerManager.getCompiler()!, caseConfig.compiler);
			},
			build: async (context: ITestContext) => {
				const compiler = getCompiler(context, name);
				if (typeof caseConfig.build === "function") {
					await caseConfig.build(context, compiler.getCompiler()!);
				} else {
					await compiler.build();
				}
			},
			run: async (env: ITestEnv, context: ITestContext) => {
				// no need to run, just check the snapshot of diagnostics
			},
			check: async (env: ITestEnv, context: ITestContext) => {
				await check(env, context, name, caseConfig.check);
			}
		});
	});
}

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
	name: string,
	custom?: (
		stats: TCompilerStats<ECompilerType.Rspack>,
		compiler: TCompiler<ECompilerType.Rspack>
	) => Promise<void>
) {
	const manager = getCompiler(context, name);
	const stats = manager.getStats()! as TCompilerStats<ECompilerType.Rspack>;
	env.expect(typeof stats).toBe("object");
	await custom?.(stats, manager.getCompiler()!);
}
