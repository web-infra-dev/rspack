import {
	IStatsAPITaskProcessorOptions,
	StatsAPITaskProcessor
} from "../processor";
import { getSimpleProcessorRunner } from "../test/simple";
import { ECompilerType } from "../type";

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
		StatsAPITaskProcessor.addSnapshotSerializer(expect);
		addedSerializer = true;
	}
	const caseConfig: TStatsAPICaseConfig = require(testConfig);
	const runner = getSimpleProcessorRunner(src, dist);

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
