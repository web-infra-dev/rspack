import { StatsCompilation } from "@rspack/core";
import checkArrayExpectation from "../helper/legacy/checkArrayExpectation";
import {
	ECompilerType,
	ITestEnv,
	ITestRunner,
	TCompilerOptions,
	TCompilerStatsCompilation
} from "../type";
import { BasicRunnerFactory } from "./basic";
import { WebRunner } from "./runner/web";

export class HotRunnerFactory<
	T extends ECompilerType
> extends BasicRunnerFactory<T> {
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
						// errorDetails: true
					});
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
				})
				.catch(callback);
		};

		return new WebRunner({
			dom: "fake",
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
	}
}
