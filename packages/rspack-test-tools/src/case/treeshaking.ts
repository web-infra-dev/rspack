import { TreeShakingProcessor } from "../processor/treeshaking";
import { BasicCaseCreator } from "../test/creator";
import { ECompilerType } from "../type";

const creator = new BasicCaseCreator({
	clean: true,
	describe: false,
	description(name, step) {
		return `${name} with newTreeshaking should match snapshot`;
	},
	steps: ({ name }) => [
		new TreeShakingProcessor({
			name,
			snapshot: "treeshaking.snap.txt",
			compilerType: ECompilerType.Rspack
		})
	]
});

export function createTreeShakingCase(name: string, src: string, dist: string) {
	creator.create(name, src, dist);
}
