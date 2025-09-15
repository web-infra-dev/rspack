import path from "node:path";
import { HotModuleReplacementPlugin } from "@rspack/core";
import { NormalProcessor } from "../processor/normal";
import { NormalRunnerFactory } from "../runner";
import { BasicCaseCreator } from "../test/creator";
import { ECompilerType } from "../type";

const NORMAL_CASES_ROOT = path.resolve(
	__dirname,
	"../../../../tests/rspack-test/normalCases"
);

const creator = new BasicCaseCreator({
	clean: true,
	describe: false,
	steps: ({ name }) => [
		new NormalProcessor({
			name,
			root: NORMAL_CASES_ROOT,
			compilerOptions: {},
			runable: true,
			compilerType: ECompilerType.Rspack
		})
	],
	runner: NormalRunnerFactory,
	concurrent: true
});

export function createNormalCase(name: string, src: string, dist: string) {
	creator.create(name, src, dist);
}

const hotCreator = new BasicCaseCreator({
	clean: true,
	describe: false,
	steps: ({ name }) => [
		new NormalProcessor({
			name,
			root: NORMAL_CASES_ROOT,
			compilerOptions: {
				plugins: [new HotModuleReplacementPlugin()]
			},
			runable: true,
			compilerType: ECompilerType.Rspack
		})
	],
	runner: NormalRunnerFactory,
	concurrent: true
});

export function createHotNormalCase(name: string, src: string, dist: string) {
	hotCreator.create(name, src, dist);
}
