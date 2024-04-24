import { ISnapshotProcessorOptions, SnapshotProcessor } from ".";
import { ECompilerType, ITestContext, TCompilerOptions } from "../type";

interface IHookProcessorOptions<T extends ECompilerType>
	extends ISnapshotProcessorOptions<T> {
	options?: (context: ITestContext) => TCompilerOptions<T>;
}

export class HookTaskProcessor extends SnapshotProcessor<ECompilerType.Rspack> {
	constructor(
		protected hookOptions: IHookProcessorOptions<ECompilerType.Rspack>
	) {
		super({
			defaultOptions: context => {
				return {
					context: context.getSource(),
					mode: "production",
					target: "async-node",
					devtool: false,
					cache: false,
					entry: "./hook",
					output: {
						path: context.getDist()
					},
					optimization: {
						minimize: false
					},
					experiments: {
						rspackFuture: {
							newTreeshaking: true
						}
					}
				};
			},
			...hookOptions,
			runable: true
		});
	}

	async config(context: ITestContext): Promise<void> {
		await super.config(context);
		const compiler = this.getCompiler(context);
		if (typeof this.hookOptions.options === "function") {
			compiler.mergeOptions(this.hookOptions.options(context));
		}
	}
}
