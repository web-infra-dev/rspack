import {
	ECompilerType,
	ITestEnv,
	ITestRunner,
	TCompilerOptions
} from "../type";
import { BasicRunnerFactory } from "./basic";
import { NormalRunner } from "./runner/normal";

export class NormalRunnerFactory<
	T extends ECompilerType
> extends BasicRunnerFactory<T> {
	protected createRunner(
		file: string,
		compilerOptions: TCompilerOptions<T>,
		env: ITestEnv
	): ITestRunner {
		return new NormalRunner({
			env,
			name: this.name,
			runInNewContext: false,
			testConfig: this.context.getTestConfig(),
			source: this.context.getSource(),
			dist: this.context.getDist(),
			compilerOptions: compilerOptions
		});
	}
}
