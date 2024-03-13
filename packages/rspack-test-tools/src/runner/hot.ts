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
		const hotUpdateContext = this.context.getValue(
			this.name,
			"hotUpdateContext"
		) as { updateIndex: number };
		return new HotRunner({
			env,
			stats: stats!,
			name: this.name,
			runInNewContext: false,
			testConfig,
			source: this.context.getSource(),
			dist: this.context.getDist(),
			compilerOptions,
			compiler,
			hotUpdateContext
		});
	}
}
