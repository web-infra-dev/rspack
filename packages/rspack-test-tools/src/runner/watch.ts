import type {
	ECompilerType,
	ITestEnv,
	ITestRunner,
	TCompilerOptions,
	TCompilerStatsCompilation
} from "../type";
import { BasicRunnerFactory } from "./basic";
import { WatchRunner } from "./runner/watch";

export class WatchRunnerFactory<
	T extends ECompilerType
> extends BasicRunnerFactory<T> {
	protected getRunnerKey(file: string): string {
		const stepName: string | void = this.context.getValue(
			this.name,
			"watchStepName"
		);
		return `${this.name}-${stepName}`;
	}
	protected createRunner(
		file: string,
		stats: TCompilerStatsCompilation<T>,
		compilerOptions: TCompilerOptions<T>,
		env: ITestEnv
	): ITestRunner {
		this.context.getCompiler<T>(this.name);
		const stepName: string | void = this.context.getValue(
			this.name,
			"watchStepName"
		);
		if (!stepName) {
			throw new Error("Can not get watch step name from context");
		}
		return new WatchRunner({
			env,
			stats,
			name: this.name,
			stepName,
			runInNewContext:
				compilerOptions.target === "web" ||
				compilerOptions.target === "webworker",
			testConfig: this.context.getTestConfig(),
			source: this.context.getSource(),
			dist: this.context.getDist(),
			compilerOptions
		});
	}
}
