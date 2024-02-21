import { runBuild, readConfigFile } from "../helper";
import { BasicRunner } from "../runner/basic";
import { EsmRunner } from "../runner/esm";
import {
	ECompilerType,
	ITestContext,
	ITestEnv,
	ITestProcessor,
	TCompiler,
	TCompilerOptions,
	TCompilerStats
} from "../type";

export interface IMultiTaskProcessorOptions<T = ECompilerType.Rspack> {
	preOptions?: (context: ITestContext) => TCompilerOptions<T>;
	postOptions?: (options: TCompilerOptions<T>, context: ITestContext) => void;
	getCompiler: () => (options: TCompilerOptions<T>) => TCompiler<T>;
	getBundle: (index: number, options: TCompilerOptions<T>) => string[] | void;
	name: string;
	configFiles?: string[];
}

export class MultiTaskProcessor<T extends ECompilerType = ECompilerType.Rspack>
	implements ITestProcessor
{
	private tasks: string[] = [];
	constructor(protected options: IMultiTaskProcessorOptions<T>) {}

	async config(context: ITestContext) {
		this.tasks = [];
		const source = context.getSource();
		const preOptions =
			typeof this.options.preOptions === "function"
				? this.options.preOptions!(context)
				: {};
		const caseOptions: TCompilerOptions<T>[] = Array.isArray(
			this.options.configFiles
		)
			? readConfigFile(source, this.options.configFiles!)
			: [{}];

		for (let [index, options] of caseOptions.entries()) {
			const taskId = `${this.options.name}[${index + 1}]`;
			context.options<T>(() => preOptions, taskId);
			context.options<T>(() => options, taskId);
			context.options<T>(options => {
				if (typeof this.options.postOptions === "function") {
					this.options.postOptions(options, context);
				}
			}, taskId);
			this.tasks.push(taskId);
		}
	}

	async compiler(context: ITestContext) {
		const factory = this.options.getCompiler();
		for (let taskId of this.tasks) {
			context.compiler<T>(options => factory(options), taskId);
		}
	}

	async build(context: ITestContext) {
		for (let taskId of this.tasks) {
			await runBuild<T>(context, taskId);
		}
	}

	async run(env: ITestEnv, context: ITestContext) {
		const compilerStats = this.getStats(context);
		for (let [index, taskId] of this.tasks.entries()) {
			context.options<T>((options: TCompilerOptions<T>) => {
				const bundles = this.options.getBundle(index, options);
				if (!bundles) {
					return;
				}
				let runner;
				if (options.target === "web" || options.target === "webworker") {
					throw new Error("TODO web runner");
				} else {
					const runnerOptions = {
						env,
						stats: compilerStats[index]!,
						name: taskId,
						runInNewContext: false,
						testConfig: {},
						source: context.getSource(),
						dist: context.getDist(),
						compilerOptions: options
					};
					if (options.experiments?.outputModule) {
						runner = new EsmRunner(runnerOptions);
					} else {
						runner = new BasicRunner(runnerOptions);
					}
				}
				for (let bundle of bundles) {
					const result = runner.run(bundle);
					// TODO: check exported result
				}
			}, taskId);
		}
	}

	getStats(context: ITestContext) {
		let res: Array<TCompilerStats<T> | null> = [];
		for (let taskId of this.tasks) {
			context.stats<T>((_, stats) => {
				res.push(stats);
			}, taskId);
		}
		return res;
	}
}
