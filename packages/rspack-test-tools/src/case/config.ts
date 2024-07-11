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
	steps: ({ name }) => [
		new ConfigProcessor({
			name,
			runable: true,
			compilerType: ECompilerType.Rspack,
			configFiles: ["rspack.config.js", "webpack.config.js"]
		})
	],
	runner: MultipleRunnerFactory
});

export function createConfigCase(name: string, src: string, dist: string) {
	creator.create(name, src, dist);
}
