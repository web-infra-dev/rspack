import {
	ECompilerType,
	ITestEnv,
	ITestRunner,
	TCompilerOptions
} from "../type";
import { WatchRunner } from "./runner/watch";
import { BasicRunnerFactory } from "./basic";

export class WatchRunnerFactory<
	T extends ECompilerType
> extends BasicRunnerFactory<T> {
	protected getRunnerKey(name: string, file: string): string {
		const stepName: string | void = this.context.getValue(
			this.name,
			"watchStepName"
		);
		return `${name}-${stepName}`;
	}
	protected createRunner(
		file: string,
		compilerOptions: TCompilerOptions<T>,
		env: ITestEnv
	): ITestRunner {
		const compiler = this.context.getCompiler<T>(this.name);
		const stepName: string | void = this.context.getValue(
			this.name,
			"watchStepName"
		);
		if (!stepName) {
			throw new Error("Can not get watch step name from context");
		}
		const stats = compiler.getStats();
		return new WatchRunner({
			env,
			stats: stats!,
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
