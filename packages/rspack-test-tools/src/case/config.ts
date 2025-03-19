import { ConfigProcessor } from "../processor/config";
import { MultipleRunnerFactory } from "../runner";
import { BasicCaseCreator } from "../test/creator";
import { ECompilerType, type TTestConfig } from "../type";

export type TConfigCaseConfig = Omit<
	TTestConfig<ECompilerType.Rspack>,
	"validate"
>;

const creator = new BasicCaseCreator({
	clean: true,
	describe: false,
	testConfig: testConfig => {
		const oldModuleScope = testConfig.moduleScope;
		testConfig.moduleScope = (ms, stats) => {
			let res = ms;
			// TODO: modify runner module scope based on stats here
			if (typeof oldModuleScope === "function") {
				res = oldModuleScope(ms, stats);
			}
			return res;
		};
	},
	steps: ({ name }) => [
		new ConfigProcessor({
			name,
			runable: true,
			compilerType: ECompilerType.Rspack,
			configFiles: ["rspack.config.js", "webpack.config.js"]
		})
	],
	runner: MultipleRunnerFactory,
	concurrent: true
});

export function createConfigCase(name: string, src: string, dist: string) {
	creator.create(name, src, dist);
}
