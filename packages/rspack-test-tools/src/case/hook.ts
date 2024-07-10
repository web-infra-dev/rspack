import path from "path";

import createLazyTestEnv from "../helper/legacy/createLazyTestEnv";
import {
	HookCasesContext,
	HookTaskProcessor,
	type IHookProcessorOptions
} from "../processor";
import { BasicRunnerFactory } from "../runner";
import { getSimpleProcessorRunner } from "../test/simple";
import { ECompilerType } from "../type";

export type THookCaseConfig = Omit<
	IHookProcessorOptions<ECompilerType.Rspack>,
	"name" | "compilerType" | "runable"
> & {
	description: string;
};

export function createHookCase(
	name: string,
	src: string,
	dist: string,
	source: string
) {
	const caseConfig: Partial<THookCaseConfig> = require(
		path.join(src, "test.js")
	);
	const testName = path.basename(
		name.slice(0, name.indexOf(path.extname(name)))
	);
	const runner = getSimpleProcessorRunner(source, dist, {
		env: () => env,
		context: () =>
			new HookCasesContext(src, testName, {
				src: source,
				dist: dist,
				runnerFactory: BasicRunnerFactory
			})
	});

	it(caseConfig.description!, async () => {
		await runner(
			name,
			new HookTaskProcessor({
				name,
				compilerType: ECompilerType.Rspack,
				findBundle: function () {
					return ["main.js"];
				},
				snapshot: path.join(src, "output.snap.txt"),
				runable: true,
				...caseConfig
			})
		);
	});
	const env = createLazyTestEnv(10000);
}
