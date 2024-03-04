import path from "path";
import { RspackNormalProcessor } from "../processor/normal";
import { BasicCaseCreator } from "../test/creator";

const creator = new BasicCaseCreator({
	clean: true,
	runable: true,
	describe: true,
	steps: ({ name }, testConfig) => [
		new RspackNormalProcessor({
			name,
			root: path.resolve(__dirname, "../../../rspack"),
			compilerOptions: {}, // do not used in rspack
			testConfig
		})
	]
});

export function createNormalCase(name: string, src: string, dist: string) {
	creator.create(name, src, dist);
}
