import path from "path";
import { RspackNormalProcessor } from "../processor/normal";
import { BasicCaseCreator } from "../test/creator";
import { NormalRunnerFactory } from "../runner";

const creator = new BasicCaseCreator({
	clean: true,
	describe: true,
	steps: ({ name }) => [
		new RspackNormalProcessor({
			name,
			root: path.resolve(__dirname, "../../tests/normalCases"),
			compilerOptions: {}, // do not used in rspack
			runable: true
		})
	],
	runner: NormalRunnerFactory
});

export function createNormalCase(name: string, src: string, dist: string) {
	creator.create(name, src, dist);
}
