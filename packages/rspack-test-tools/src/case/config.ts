import { RspackConfigProcessor } from "../processor/config";
import { MultipleRunnerFactory } from "../runner";
import { BasicCaseCreator } from "../test/creator";

const creator = new BasicCaseCreator({
	clean: true,
	describe: true,
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
