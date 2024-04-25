import { ECompilerType } from "../type";
import path from "path";
import { getSimpleProcessorRunner } from "../test/simple";
import { StatsAPITaskProcessor } from "../processor";

let addedSerializer = false;

export function createStatsAPICase(
	name: string,
	src: string,
	dist: string,
	root: string
) {
	if (!addedSerializer) {
		StatsAPITaskProcessor.addSnapshotSerializer();
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
			new StatsAPITaskProcessor({
				name: name,
				compilerType: ECompilerType.Rspack,
				...caseConfig
			})
		);
	});
}
