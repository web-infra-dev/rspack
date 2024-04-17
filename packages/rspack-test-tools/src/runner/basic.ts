import {
	ECompilerType,
	ITestContext,
	ITestEnv,
	ITestRunner,
	TCompilerOptions,
	TCompilerStatsCompilation,
	TRunnerFactory
} from "../type";
import { WebRunner } from "./runner/web";
import { EsmRunner } from "./runner/esm";

export class BasicRunnerFactory<T extends ECompilerType>
	implements TRunnerFactory<T>
{
	constructor(
		protected name: string,
		protected context: ITestContext
	) {}

	create(
		file: string,
		compilerOptions: TCompilerOptions<T>,
		env: ITestEnv
	): ITestRunner {
		const key = this.getRunnerKey(file);
		const exists = this.context.getRunner(key);
		if (exists) {
			return exists;
		}
		const compiler = this.context.getCompiler<T>(this.name);
		const stats = compiler.getStats()!.toJson({
			errorDetails: true
		});
		const runner = this.createRunner(file, stats, compilerOptions, env);
		this.context.setRunner(key, runner);
		return runner;
	}

	protected getRunnerKey(file: string) {
		return this.name;
	}

	protected createRunner(
		file: string,
		stats: TCompilerStatsCompilation<T>,
		compilerOptions: TCompilerOptions<T>,
		env: ITestEnv
	): ITestRunner {
		const runnerOptions = {
			env,
			stats,
			name: this.name,
			testConfig: this.context.getTestConfig(),
			source: this.context.getSource(),
			dist: this.context.getDist(),
			compilerOptions
		};
		if (
			compilerOptions.target === "web" ||
			compilerOptions.target === "webworker"
		) {
			return new WebRunner<T>(runnerOptions);
		} else {
			return new EsmRunner<T>(runnerOptions);
		}
	}
}
