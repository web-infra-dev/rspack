import { EsmOutputProcessor } from "../processor/esm-output";
import { BasicRunnerFactory } from "../runner";
import { BasicCaseCreator } from "../test/creator";
import { ECompilerType } from "../type";

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
		new EsmOutputProcessor({
			name,
			runable: true,
			compilerType: ECompilerType.Rspack,
			configFiles: [
				"rspack.config.cjs",
				"rspack.config.js",
				"webpack.config.js"
			],
			snapshot: "esm.snap.txt"
		})
	],
	runner: BasicRunnerFactory,
	concurrent: true
});

export function createEsmOutputCase(name: string, src: string, dist: string) {
	creator.create(name, src, dist);
}
