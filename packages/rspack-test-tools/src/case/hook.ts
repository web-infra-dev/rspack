import path from "path";
import { getSimpleProcessorRunner } from "../test/simple";
import { HookCasesContext, HookTaskProcessor } from "../processor";
import { BasicRunnerFactory } from "../runner";
import { ECompilerType, TCompilerOptions } from "../type";
import createLazyTestEnv from "../helper/legacy/createLazyTestEnv";

export function createHookCase(
	name: string,
	src: string,
	dist: string,
	source: string
) {
	const caseConfig = require(path.join(src, "test.js"));
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

	it(caseConfig.description, async () => {
		await runner(
			name,
			new HookTaskProcessor({
				name,
				compilerType: ECompilerType.Rspack,
				findBundle: function (
					i: number,
					options: TCompilerOptions<ECompilerType.Rspack>
				) {
					return ["main.js"];
				},
				snapshot: path.join(src, "output.snap.txt"),
				...caseConfig
			})
		);
	});
	const env = createLazyTestEnv(10000);
}
