import { RspackConfigProcessor } from "../processor/config";
import { BasicCaseCreator } from "../test/creator";

const creator = new BasicCaseCreator({
	clean: true,
	runable: true,
	describe: true,
	steps: ({ name }, testConfig) => [
		new RspackConfigProcessor({
			name,
			testConfig
		})
	]
});

export function createConfigCase(name: string, src: string, dist: string) {
	creator.create(name, src, dist);
}
