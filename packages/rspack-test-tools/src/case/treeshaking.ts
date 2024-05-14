import { RspackTreeShakingProcessor } from "../processor/treeshaking";
import { BasicCaseCreator } from "../test/creator";

const creator = new BasicCaseCreator({
	clean: true,
	describe: false,
	description(name, step) {
		if (step === 0) {
			return `${name} with builtin.treeShaking should match snapshot`;
		} else {
			return `${name} with newTreeshaking should match snapshot`;
		}
	},
	steps: ({ name }) => [
		new RspackTreeShakingProcessor({
			name,
			snapshot: "output.snap.txt",
			type: "builtin"
		}),
		new RspackTreeShakingProcessor({
			name,
			snapshot: "new_treeshaking.snap.txt",
			type: "new"
		})
	]
});

export function createTreeShakingCase(name: string, src: string, dist: string) {
	creator.create(name, src, dist);
}
