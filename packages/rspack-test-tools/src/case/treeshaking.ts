import { RspackTreeShakingProcessor } from "../processor/treeshaking";
import { BasicCaseCreator } from "../test/creator";

const creator = new BasicCaseCreator({
	clean: true,
	describe: true,
	steps: ({ name }) => [
		new RspackTreeShakingProcessor({
			name,
			snapshot: "new_treeshaking.snap"
		})
	]
});

export function createTreeShakingCase(name: string, src: string, dist: string) {
	creator.create(name, src, dist);
}
