import { StatsCompilation } from "@rspack/core";
import checkArrayExpectation from "../helper/legacy/checkArrayExpectation";
import {
	ECompilerType,
	ITestEnv,
	ITestRunner,
	TCompilerOptions,
	TCompilerStats,
	TCompilerStatsCompilation
} from "../type";
import { HotRunner } from "./runner/hot";
import { HotRunnerFactory } from "./hot";

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
			callback: (error: Error | null, stats?: StatsCompilation) => void
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
							stats: TCompilerStats<T>
						) => void;
						checker(hotUpdateContext, stats as TCompilerStats<T>);
						callback(null, jsonStats as StatsCompilation);
					} catch (e) {
						callback(e as Error);
					}
				})
				.catch(callback);
		};

		return new HotRunner({
			env,
			stats,
			name: this.name,
			runInNewContext: false,
			testConfig,
			source,
			dist,
			next,
			compilerOptions
		});
	}
}
