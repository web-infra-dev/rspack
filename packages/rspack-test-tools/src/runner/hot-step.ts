import { StatsCompilation } from "@rspack/core";

import {
	ECompilerType,
	ITestEnv,
	ITestRunner,
	TCompilerOptions,
	TCompilerStats,
	TCompilerStatsCompilation
} from "../type";
import { HotRunnerFactory } from "./hot";
import { WebRunner } from "./runner/web";
import { THotStepRuntimeData } from "./type";

export class HotStepRunnerFactory<
	T extends ECompilerType
> extends HotRunnerFactory<T> {
	protected createRunner(
		file: string,
		stats: TCompilerStatsCompilation<T>,
		compilerOptions: TCompilerOptions<T>,
		env: ITestEnv
	): ITestRunner {
		const compiler = this.context.getCompiler(this.name);
		const testConfig = this.context.getTestConfig();
		const source = this.context.getSource();
		const dist = this.context.getDist();
		const hotUpdateContext = this.context.getValue(
			this.name,
			"hotUpdateContext"
		) as { updateIndex: number };

		const next = (
			callback: (
				error: Error | null,
				stats?: TCompilerStatsCompilation<T>
			) => void
		) => {
			hotUpdateContext.updateIndex++;
			compiler
				.build()
				.then(stats => {
					if (!stats)
						return callback(new Error("Should generate stats during build"));
					const jsonStats = stats.toJson({
						errorDetails: true
					});
					try {
						const checker = this.context.getValue(
							this.name,
							jsonStats.errors?.length
								? "hotUpdateStepErrorChecker"
								: "hotUpdateStepChecker"
						) as (
							context: { updateIndex: number },
							stats: TCompilerStats<T>,
							runtime: THotStepRuntimeData
						) => void;
						checker(
							hotUpdateContext,
							stats as TCompilerStats<T>,
							runner.getGlobal("__HMR_UPDATED_RUNTIME__") as THotStepRuntimeData
						);
						callback(null, jsonStats as StatsCompilation);
					} catch (e) {
						callback(e as Error);
					}
				})
				.catch(callback);
		};

		const runner = new WebRunner({
			dom: "jsdom",
			env,
			stats,
			name: this.name,
			runInNewContext: false,
			testConfig: {
				...testConfig,
				moduleScope(ms) {
					if (typeof testConfig.moduleScope === "function") {
						ms = testConfig.moduleScope(ms);
					}
					ms["NEXT"] = next;
					return ms;
				}
			},
			source,
			dist,
			compilerOptions
		});

		return runner;
	}
}
