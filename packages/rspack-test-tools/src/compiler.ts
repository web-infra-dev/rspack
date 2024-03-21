import EventEmitter from "events";
import {
	ECompilerType,
	ITestCompilerManager,
	TCompiler,
	TCompilerFactory,
	TCompilerOptions,
	TCompilerStats
} from "./type";
import merge from "webpack-merge";

export const enum ECompilerEvent {
	Build = "build",
	Option = "option",
	Create = "create",
	Close = "close"
}

export const COMPILER_FACTORIES: Record<
	ECompilerType,
	TCompilerFactory<ECompilerType>
> = {
	[ECompilerType.Rspack]: ((options: TCompilerOptions<ECompilerType.Rspack>) =>
		require("@rspack/core")(options)) as TCompilerFactory<ECompilerType>,
	[ECompilerType.Webpack]: ((
		options: TCompilerOptions<ECompilerType.Webpack>
	) => require("webpack")(options)) as TCompilerFactory<ECompilerType>
};
export class TestCompilerManager<T extends ECompilerType>
	implements ITestCompilerManager<T>
{
	protected compilerOptions: TCompilerOptions<T> = {} as TCompilerOptions<T>;
	protected compilerInstance: TCompiler<T> | null = null;
	protected compilerStats: TCompilerStats<T> | null = null;
	protected emitter: EventEmitter = new EventEmitter();

	constructor(protected type: T) {}

	getOptions(): TCompilerOptions<T> {
		return this.compilerOptions;
	}

	setOptions(newOptions: TCompilerOptions<T>): TCompilerOptions<T> {
		this.compilerOptions = newOptions;
		this.emitter.emit(ECompilerEvent.Option, this.compilerOptions);
		return this.compilerOptions;
	}

	mergeOptions(newOptions: TCompilerOptions<T>): TCompilerOptions<T> {
		this.compilerOptions = merge(this.compilerOptions, newOptions);
		this.emitter.emit(ECompilerEvent.Option, this.compilerOptions);
		return this.compilerOptions;
	}

	getCompiler(): TCompiler<T> | null {
		return this.compilerInstance;
	}

	createCompiler(): TCompiler<T> {
		this.compilerInstance = COMPILER_FACTORIES[this.type](
			this.compilerOptions
		) as TCompiler<T>;
		this.emitter.emit(ECompilerEvent.Create, this.compilerInstance);
		return this.compilerInstance;
	}

	build(): Promise<TCompilerStats<T>> {
		if (!this.compilerInstance)
			throw new Error("Compiler should be created before build");
		return new Promise<TCompilerStats<T>>((resolve, reject) => {
			this.compilerInstance!.run((error, newStats) => {
				this.emitter.emit(ECompilerEvent.Build, error, newStats);
				if (error) return reject(error);
				this.compilerStats = newStats as TCompilerStats<T>;
				resolve(newStats as TCompilerStats<T>);
			});
		});
	}

	watch(timeout: number = 1000) {
		if (!this.compilerInstance)
			throw new Error("Compiler should be created before watch");
		this.compilerInstance!.watch(
			{
				aggregateTimeout: timeout
			},
			(error, newStats) => {
				this.emitter.emit(ECompilerEvent.Build, error, newStats);
				if (error) return error;
				if (newStats) {
					this.compilerStats = newStats as TCompilerStats<T>;
				}
				return newStats;
			}
		);
	}

	getStats() {
		return this.compilerStats;
	}

	getEmitter() {
		return this.emitter;
	}

	close(): Promise<void> {
		return new Promise<void>((resolve, reject) => {
			if (this.compilerInstance) {
				this.compilerInstance.close(e => {
					this.emitter.emit(ECompilerEvent.Close, e);
					e ? reject(e) : resolve();
				});
			} else {
				resolve();
			}
		});
	}
}
