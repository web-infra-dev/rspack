import path from "node:path";
import { HotModuleReplacementPlugin } from "@rspack/core";
import { NormalProcessor } from "../processor/normal";
import { NormalRunnerFactory } from "../runner";
import {
	BasicCaseCreator,
	type IBasicCaseCreatorOptions
} from "../test/creator";
import { ECompilerType } from "../type";

const NORMAL_CASES_ROOT = path.resolve(
	__dirname,
	"../../../../tests/rspack-test/normalCases"
);

const createCaseOptions = (
	hot: boolean
): IBasicCaseCreatorOptions<ECompilerType> => {
	return {
		clean: true,
		describe: false,
		steps: ({ name }) => [
			new NormalProcessor({
				name,
				root: NORMAL_CASES_ROOT,
				compilerOptions: {
					plugins: hot ? [new HotModuleReplacementPlugin()] : []
				},
				runable: true,
				compilerType: ECompilerType.Rspack
			})
		],
		runner: NormalRunnerFactory,
		concurrent: true
	};
};

const creator = new BasicCaseCreator(createCaseOptions(false));
export function createNormalCase(name: string, src: string, dist: string) {
	creator.create(name, src, dist);
}

const hotCreator = new BasicCaseCreator(createCaseOptions(true));
export function createHotNormalCase(name: string, src: string, dist: string) {
	hotCreator.create(name, src, dist);
}
