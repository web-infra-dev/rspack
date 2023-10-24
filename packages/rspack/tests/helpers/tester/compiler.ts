import {
	ECompilerType,
	ITestCompilerManager,
	ITestContext,
	TCompiler,
	TCompilerOptions,
	TCompilerStats
} from "./type";

export class TestCompilerManager<T extends ECompilerType>
	implements ITestCompilerManager<T>
{
	private options: TCompilerOptions<T> = {} as TCompilerOptions<T>;
	private compiler: TCompiler<T> | null = null;
	private stats: TCompilerStats<T> | null = null;
	private result: unknown;

	setOptions(
		context: ITestContext,
		fn: (options: TCompilerOptions<T>) => TCompilerOptions<T>
	) {
		try {
			this.options = fn(this.options);
		} catch (e) {
			context.emitError(e);
		}
	}
	setCompiler(
		context: ITestContext,
		fn: (
			options: TCompilerOptions<T>,
			compiler: TCompiler<T> | null
		) => TCompiler<T> | null
	) {
		try {
			this.compiler = fn(this.options, this.compiler);
		} catch (e) {
			context.emitError(e);
		}
	}
	setStats(
		context: ITestContext,
		fn: (
			compiler: TCompiler<T> | null,
			stats: TCompilerStats<T> | null
		) => TCompilerStats<T> | null
	) {
		try {
			this.stats = fn(this.compiler, this.stats);
		} catch (e) {
			context.emitError(e);
		}
	}
	setResult(
		context: ITestContext,
		fn: <R>(compiler: TCompiler<T> | null, result: R) => R
	) {
		try {
			this.result = fn(this.compiler, this.result);
		} catch (e) {
			context.emitError(e);
		}
	}
	async build(
		context: ITestContext,
		fn: (compiler: TCompiler<T>) => Promise<void>
	) {
		if (!this.compiler) {
			context.emitError(new Error("Build failed: compiler not exists"));
		}
		try {
			await fn(this.compiler!);
		} catch (e) {
			context.emitError(e);
		}
	}
}
