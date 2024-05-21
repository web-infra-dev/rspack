import { StatsProcessor } from "../processor/stats";
import { BasicCaseCreator } from "../test/creator";
import { ECompilerType } from "../type";

const creator = new BasicCaseCreator({
	clean: true,
	describe: false,
	steps: ({ name }) => [
		new StatsProcessor({
			name,
			compilerType: ECompilerType.Rspack,
			configFiles: ["rspack.config.js", "webpack.config.js"]
		})
	],
	description: () => `should print correct stats for`
});

export function createStatsOutputCase(name: string, src: string, dist: string) {
	creator.create(name, src, dist);
}
