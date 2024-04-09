import { StatsCompilation } from "@rspack/core";
import checkArrayExpectation from "../helper/legacy/checkArrayExpectation";
import {
	ECompilerType,
	ITestEnv,
	ITestRunner,
	TCompilerOptions
} from "../type";
import { BasicRunnerFactory } from "./basic";
import { HotRunner } from "./runner/hot";

export class HotRunnerFactory<
	T extends ECompilerType
> extends BasicRunnerFactory<T> {
	protected createRunner(
		file: string,
		compilerOptions: TCompilerOptions<T>,
		env: ITestEnv
	): ITestRunner {
		const compiler = this.context.getCompiler(this.name);
		const testConfig = this.context.getTestConfig();
		const stats = compiler.getStats();
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

		return new HotRunner({
			env,
			stats: stats!,
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
