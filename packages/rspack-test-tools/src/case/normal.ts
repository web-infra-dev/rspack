import path from "path";

import { RspackNormalProcessor } from "../processor/normal";
import { NormalRunnerFactory } from "../runner";
import { BasicCaseCreator } from "../test/creator";

const creator = new BasicCaseCreator({
	clean: true,
	describe: false,
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
