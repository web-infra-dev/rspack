import { MultiTaskProcessor } from "../processor";
import { MultipleRunnerFactory } from "../runner";
import { BasicCaseCreator } from "../test/creator";
import { ECompilerType, type TTestConfig } from "../type";
import { defaultOptions, findBundle, overrideOptions } from "./config";

export type TSerialCaseConfig = Omit<
	TTestConfig<ECompilerType.Rspack>,
	"validate"
>;

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
	steps: ({ name }) => [
		new MultiTaskProcessor({
			name,
			runable: true,
			compilerType: ECompilerType.Rspack,
			configFiles: ["rspack.config.js", "webpack.config.js"],
			defaultOptions,
			overrideOptions,
			findBundle
		})
	],
	runner: MultipleRunnerFactory
});

export function createSerialCase(name: string, src: string, dist: string) {
	creator.create(name, src, dist);
}
