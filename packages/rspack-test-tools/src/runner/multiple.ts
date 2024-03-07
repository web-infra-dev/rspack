import {
	ECompilerType,
	ITestEnv,
	ITestRunner,
	TCompilerOptions
} from "../type";
import { BasicRunnerFactory } from "./basic";

export class MultipleRunnerFactory<
	T extends ECompilerType
> extends BasicRunnerFactory<T> {
	protected getRunnerKey(name: string, file: string): string {
		const multiFileIndexMap: Record<string, number> =
			this.context.getValue(this.name, "multiFileIndexMap") || {};
		return `${name}-${multiFileIndexMap[file]}`;
	}

	protected createRunner(
		file: string,
		compilerOptions: TCompilerOptions<T>,
		env: ITestEnv
	): ITestRunner {
		const multiCompilerOptions: TCompilerOptions<T>[] =
			this.context.getValue(this.name, "multiCompilerOptions") || [];
		const multiFileIndexMap: Record<string, number> =
			this.context.getValue(this.name, "multiFileIndexMap") || {};
		if (typeof multiFileIndexMap[file] === "undefined") {
			throw new Error("Unexpect file in multiple runner");
		}
		return super.createRunner(
			file,
			multiCompilerOptions[multiFileIndexMap[file]],
			env
		);
	}
}
