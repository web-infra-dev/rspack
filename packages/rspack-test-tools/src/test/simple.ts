import { ITestEnv, ITestProcessor } from "../type";
import { TestContext } from "./context";

const CONTEXT_MAP: Map<
	string,
	(name: string, processor: ITestProcessor) => Promise<void>
> = new Map();

export function getSimpleProcessorRunner(
	src: string,
	dist: string,
	env: ITestEnv
) {
	const key = `src: ${src}, dist: ${dist}`;
	if (!CONTEXT_MAP.has(key)) {
		const context = new TestContext({
			src,
			dist
		});
		const runner = async function run(name: string, processor: ITestProcessor) {
			try {
				await processor.before?.(context);
				await processor.config?.(context);
				await processor.compiler?.(context);
				await processor.build?.(context);
			} catch (e: unknown) {
				context.emitError(name, e as Error);
			} finally {
				await processor.run?.(env, context);
				await processor.check?.(env, context);
				await processor.after?.(context);
			}
		};
		CONTEXT_MAP.set(key, runner);
	}
	return CONTEXT_MAP.get(key)!;
}
