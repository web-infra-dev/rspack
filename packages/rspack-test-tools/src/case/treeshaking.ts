import type { RspackOptions } from "@rspack/core";
import { BasicCaseCreator } from "../test/creator";
import type { ITestContext, ITestEnv } from "../type";
import { defaultOptions } from "./builtin";
import { build, checkSnapshot, compiler } from "./common";

const creator = new BasicCaseCreator({
	clean: true,
	describe: false,
	description(name, step) {
		return `${name} with newTreeshaking should match snapshot`;
	},
	steps: ({ name }) => [
		{
			config: async (context: ITestContext) => {
				const compiler = context.getCompiler();
				const options = defaultOptions(context);
				overrideOptions(context, options);
				compiler.setOptions(options);
			},
			compiler: async (context: ITestContext) => {
				await compiler(context, name);
			},
			build: async (context: ITestContext) => {
				await build(context, name);
			},
			run: async (env: ITestEnv, context: ITestContext) => {
				// no need to run, just check snapshot
			},
			check: async (env: ITestEnv, context: ITestContext) => {
				await checkSnapshot(env, context, name, "treeshaking.snap.txt");
			}
		}
	]
});

export function createTreeShakingCase(name: string, src: string, dist: string) {
	creator.create(name, src, dist);
}

function overrideOptions(context: ITestContext, options: RspackOptions) {
	options.target = options.target || ["web", "es2022"];
	options.optimization ??= {};
	options.optimization.providedExports = true;
	options.optimization.innerGraph = true;
	options.optimization.usedExports = true;

	if (!global.printLogger) {
		options.infrastructureLogging = {
			level: "error"
		};
	}
}
