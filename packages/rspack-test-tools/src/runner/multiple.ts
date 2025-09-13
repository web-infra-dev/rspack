import type {
	ECompilerType,
	ITestEnv,
	ITestRunner,
	TCompilerOptions,
	TCompilerStatsCompilation
} from "../type";
import { BasicRunnerFactory } from "./basic";

export class MultipleRunnerFactory<
	T extends ECompilerType
> extends BasicRunnerFactory<T> {
	protected runned: Set<string> = new Set();
	protected getRunnerKey(file: string): string {
		const { getIndex } = this.getFileIndexHandler(file);
		const [index, seq] = getIndex();
		return `${this.name}-${index}[${seq}]`;
	}

	protected createRunner(
		file: string,
		stats: () => TCompilerStatsCompilation<T>,
		compilerOptions: TCompilerOptions<T>,
		env: ITestEnv
	): ITestRunner {
		const { getIndex, flagIndex } = this.getFileIndexHandler(file);
		const key = this.getRunnerKey(file);
		const exists = this.context.getRunner(key);
		if (exists) {
			flagIndex();
			return exists;
		}
		const multiCompilerOptions: TCompilerOptions<T>[] =
			this.context.getValue(this.name, "multiCompilerOptions") || [];
		const [index] = getIndex();
		const runner = super.createRunner(
			file,
			() => {
				const s = stats();
				if (s.children?.length && s.children.length > 1) {
					s.__index__ = index;
					return s;
				}
				return s.children![index];
			},
			multiCompilerOptions[index],
			env
		);
		flagIndex();
		this.context.setRunner(key, runner);
		return runner;
	}

	protected getFileIndexHandler(file: string) {
		const multiFileIndexMap: Record<string, number[]> =
			this.context.getValue(this.name, "multiFileIndexMap") || {};
		if (typeof multiFileIndexMap[file] === "undefined") {
			throw new Error("Unexpect file in multiple runner");
		}
		const indexList = multiFileIndexMap[file];
		const seq = indexList.findIndex(
			(index, n) => !this.runned.has(`${this.name}:${file}[${n}]`)
		);
		if (seq === -1) {
			throw new Error(`File ${file} should run only ${indexList.length} times`);
		}
		const getIndex = () => [indexList[seq], seq];
		const flagIndex = () => this.runned.add(`${this.name}:${file}[${seq}]`);
		return { getIndex, flagIndex };
	}
}
