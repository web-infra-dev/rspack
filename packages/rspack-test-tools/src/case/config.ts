import { ECompilerType, TTestConfig } from "..";
import { RspackConfigProcessor } from "../processor/config";
import { MultipleRunnerFactory } from "../runner";
import { BasicCaseCreator } from "../test/creator";

export type TConfigCaseConfig = Omit<
	TTestConfig<ECompilerType.Rspack>,
	"validate"
>;

const creator = new BasicCaseCreator({
	clean: true,
	describe: false,
	steps: ({ name }) => [
		new RspackConfigProcessor({
			name,
			runable: true
		})
	],
	runner: MultipleRunnerFactory
});

export function createConfigCase(name: string, src: string, dist: string) {
	creator.create(name, src, dist);
}
