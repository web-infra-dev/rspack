import path from "path";

import { NormalProcessor } from "../processor/normal";
import { NormalRunnerFactory } from "../runner";
import { BasicCaseCreator } from "../test/creator";
import { ECompilerType } from "../type";

const creator = new BasicCaseCreator({
	clean: true,
	describe: false,
	steps: ({ name }) => [
		new NormalProcessor({
			name,
			root: path.resolve(__dirname, "../../tests/normalCases"),
			compilerOptions: {}, // do not used in rspack
			runable: true,
			compilerType: ECompilerType.Rspack
		})
	],
	runner: NormalRunnerFactory
});

export function createNormalCase(name: string, src: string, dist: string) {
	creator.create(name, src, dist);
}
