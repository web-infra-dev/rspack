import { BasicCaseCreator } from "../test/creator";
import type { TTestConfig } from "../type";
import { createConfigProcessor } from "./config";
import { createMultiCompilerRunner, getMultiCompilerRunnerKey } from "./runner";

export type TSerialCaseConfig = Omit<TTestConfig, "validate">;

const creator = new BasicCaseCreator({
	clean: true,
	describe: false,
	testConfig: testConfig => {
		const oldModuleScope = testConfig.moduleScope;
		testConfig.moduleScope = (ms, stats, compilerOptions) => {
			let res = ms;
			// TODO: modify runner module scope based on stats here
			if (typeof oldModuleScope === "function") {
				res = oldModuleScope(ms, stats, compilerOptions);
			}
			return res;
		};
	},
	steps: ({ name }) => [createConfigProcessor(name)],
	runner: {
		key: getMultiCompilerRunnerKey,
		runner: createMultiCompilerRunner
	},
	concurrent: false
});

export function createSerialCase(name: string, src: string, dist: string) {
	creator.create(name, src, dist);
}
