import { RspackStatsProcessor } from "../processor/stats";
import { BasicCaseCreator } from "../test/creator";

const creator = new BasicCaseCreator({
	clean: true,
	describe: false,
	steps: ({ name }) => [
		new RspackStatsProcessor({
			name
		})
	],
	description: name => `should print correct stats for ${name}`
});

export function createStatsOutputCase(name: string, src: string, dist: string) {
	creator.create(name, src, dist);
}
