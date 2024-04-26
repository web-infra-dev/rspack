import { ECompilerType } from "../type";
import path from "path";
import { getSimpleProcessorRunner } from "../test/simple";
import { ErrorTaskProcessor } from "../processor";

let addedSerializer = false;

export function createErrorCase(
	name: string,
	src: string,
	dist: string,
	root: string
) {
	if (!addedSerializer) {
		ErrorTaskProcessor.addSnapshotSerializer();
		addedSerializer = true;
	}
	const caseConfig = require(path.join(root, name));
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
