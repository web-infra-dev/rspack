import { RspackTreeShakingProcessor } from "../processor/treeshaking";
import { BasicCaseCreator } from "../test/creator";

const creator = new BasicCaseCreator({
	clean: true,
	describe: false,
	description(name, step) {
		return `${name} with newTreeshaking should match snapshot`;
	},
	steps: ({ name }) => [
		new RspackTreeShakingProcessor({
			name,
			snapshot: "treeshaking.snap.txt"
		})
	]
});

export function createTreeShakingCase(name: string, src: string, dist: string) {
	creator.create(name, src, dist);
}
