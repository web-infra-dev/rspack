import { RspackBuiltinProcessor } from "../processor";
import { BasicCaseCreator } from "../test/creator";

const creator = new BasicCaseCreator({
	clean: true,
	describe: true,
	description(name) {
		return `${name} should match snapshot`;
	},
	steps: ({ name }) => [
		new RspackBuiltinProcessor({
			name,
			snapshot: "output.snap.txt"
		})
	]
});

export function createBuiltinCase(name: string, src: string, dist: string) {
	creator.create(name, src, dist);
}
