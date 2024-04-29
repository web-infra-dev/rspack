import { ECompilerType } from "../type";
import { getSimpleProcessorRunner } from "../test/simple";
import { ErrorTaskProcessor } from "../processor";

let addedSerializer = false;

export function createErrorCase(
	name: string,
	src: string,
	dist: string,
	testConfig: string
) {
	if (!addedSerializer) {
		ErrorTaskProcessor.addSnapshotSerializer();
		addedSerializer = true;
	}
	const caseConfig = require(testConfig);
	const runner = getSimpleProcessorRunner(src, dist, {
		it,
		beforeEach,
		afterEach
	});

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
