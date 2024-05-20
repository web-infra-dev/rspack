import { ErrorTaskProcessor, IErrorTaskProcessorOptions } from "../processor";
import { getSimpleProcessorRunner } from "../test/simple";
import { ECompilerType } from "../type";

let addedSerializer = false;

export type TErrorCaseConfig = Omit<
	IErrorTaskProcessorOptions<ECompilerType.Rspack>,
	"name" | "compilerType"
> & {
	description: string;
};

export function createErrorCase(
	name: string,
	src: string,
	dist: string,
	testConfig: string
) {
	if (!addedSerializer) {
		ErrorTaskProcessor.addSnapshotSerializer(expect);
		addedSerializer = true;
	}
	const caseConfig = require(testConfig);
	const runner = getSimpleProcessorRunner(src, dist);

	it(caseConfig.description, async () => {
		await runner(
			name,
			new ErrorTaskProcessor({
				name: name,
				compilerType: ECompilerType.Rspack,
				...caseConfig
			})
		);
	});
}
