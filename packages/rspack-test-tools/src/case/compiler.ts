import { ISimpleProcessorOptions, SimpleTaskProcessor } from "../processor";
import { getSimpleProcessorRunner } from "../test/simple";
import { ECompilerType } from "../type";

export type TCompilerCaseConfig = Omit<
	ISimpleProcessorOptions,
	"name" | "compilerType"
> & {
	description: string;
};

export function createCompilerCase(
	name: string,
	src: string,
	dist: string,
	testConfig: string
) {
	const caseConfig: TCompilerCaseConfig = require(testConfig);
	const runner = getSimpleProcessorRunner(src, dist);

	it(caseConfig.description, async () => {
		await runner(
			name,
			new SimpleTaskProcessor({
				name: name,
				compilerType: ECompilerType.Rspack,
				...caseConfig
			})
		);
	});
}
