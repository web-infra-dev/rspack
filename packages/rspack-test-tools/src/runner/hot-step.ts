import { StatsCompilation } from "@rspack/core";

import checkArrayExpectation from "../helper/legacy/checkArrayExpectation";
import {
	ECompilerType,
	ITestEnv,
	ITestRunner,
	TCompilerOptions,
	TCompilerStats,
	TCompilerStatsCompilation,
	TUpdateOptions
} from "../type";
import { HotRunnerFactory } from "./hot";
import { WebRunner } from "./runner/web";
import { THotStepRuntimeData } from "./type";

declare var global: {
	__CHANGED_FILES__: Map<string, number>;
};

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
		const hotUpdateContext = this.context.getValue<TUpdateOptions>(
			this.name,
			"hotUpdateContext"
		)!;

		const next = (
			callback: (
				error: Error | null,
				stats?: TCompilerStatsCompilation<T>
			) => void
		) => {
			hotUpdateContext.updateIndex++;
			// TODO: find a better way to collect changed files from fake-update-loader
			const changedFiles = new Map();
			global["__CHANGED_FILES__"] = changedFiles;
			compiler
				.build()
				.then(stats => {
					if (!stats)
						return callback(new Error("Should generate stats during build"));

					const jsonStats = stats.toJson({
						errorDetails: true
					});

					hotUpdateContext.totalUpdates = Math.max(
						hotUpdateContext.totalUpdates,
						...changedFiles.values()
					);
					hotUpdateContext.changedFiles = [...changedFiles.keys()];

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
						if (
							checkArrayExpectation(
								source,
								jsonStats,
								"error",
								"errors" + hotUpdateContext.updateIndex,
								"Error",
								callback
							)
						) {
							return;
						}
						if (
							checkArrayExpectation(
								source,
								jsonStats,
								"warning",
								"warnings" + hotUpdateContext.updateIndex,
								"Warning",
								callback
							)
						) {
							return;
						}
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
