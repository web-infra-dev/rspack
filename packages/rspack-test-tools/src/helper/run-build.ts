import EventEmitter from "events";
import { ECompilerType, ITestContext, TCompilerStats } from "../type";

export async function runBuild<T extends ECompilerType>(
	context: ITestContext,
	name?: string
): Promise<TCompilerStats<T> | null> {
	let stats: TCompilerStats<T> | null = null;
	await context.build<T>(
		compiler =>
			new Promise<void>((resolve, reject) => {
				compiler.run((error, newStats) => {
					if (error) return reject(error);
					if (newStats) {
						context.stats(() => newStats as TCompilerStats<T>, name);
					}
					stats = newStats as TCompilerStats<T>;
					resolve();
				});
			}),
		name
	);
	return stats;
}

export function startWatch<T extends ECompilerType>(
	context: ITestContext,
	aggregateTimeout: number,
	emitter: EventEmitter,
	name?: string
): void {
	let stats: TCompilerStats<T> | null = null;
	context.compiler<T>((_, compiler) => {
		if (!compiler) {
			emitter.emit("built", new Error("Compiler not exists when start watch"));
			return;
		}
		compiler.watch(
			{
				aggregateTimeout
			},
			(error, newStats) => {
				if (error) return emitter.emit("built", error);
				if (newStats) {
					context.stats(() => newStats as TCompilerStats<T>, name);
				}
				return emitter.emit("built", null, newStats);
			}
		);
	}, name);
}
