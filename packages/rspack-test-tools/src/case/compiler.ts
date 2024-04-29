import { ECompilerType } from "../type";
import path from "path";
import { SimpleTaskProcessor } from "../processor";
import { getSimpleProcessorRunner } from "../test/simple";

export function createCompilerCase(
	name: string,
	src: string,
	dist: string,
	testConfig: string
) {
	const caseConfig = require(testConfig);

	const runner = getSimpleProcessorRunner(src, dist, {
		it,
		beforeEach,
		afterEach
	});

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
