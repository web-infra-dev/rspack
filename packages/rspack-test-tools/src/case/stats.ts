import { RspackStatsProcessor } from "../processor/stats";
import { BasicCaseCreator } from "../test/creator";

const creator = new BasicCaseCreator({
	clean: true,
	runable: false,
	describe: false,
	steps: ({ name }, testConfig) => [
		new RspackStatsProcessor({
			name,
			testConfig
		})
	],
	description: name => `should print correct stats for ${name}`
});

export function createStatsCase(name: string, src: string, dist: string) {
	creator.create(name, src, dist);
}
