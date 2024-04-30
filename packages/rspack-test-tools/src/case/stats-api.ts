import { ECompilerType } from "../type";
import { getSimpleProcessorRunner } from "../test/simple";
import {
	IStatsAPITaskProcessorOptions,
	StatsAPITaskProcessor
} from "../processor";

let addedSerializer = false;

export type TStatsAPICaseConfig = Omit<
	IStatsAPITaskProcessorOptions<ECompilerType.Rspack>,
	"name" | "compilerType"
> & {
	description: string;
};

export function createStatsAPICase(
	name: string,
	src: string,
	dist: string,
	testConfig: string
) {
	if (!addedSerializer) {
		StatsAPITaskProcessor.addSnapshotSerializer();
		addedSerializer = true;
	}
	const caseConfig: TStatsAPICaseConfig = require(testConfig);
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
