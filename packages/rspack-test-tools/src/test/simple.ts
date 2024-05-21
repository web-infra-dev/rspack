import { ITestContext, ITestEnv, ITestProcessor } from "../type";
import { TestContext } from "./context";

const CONTEXT_MAP: Map<
	string,
	(name: string, processor: ITestProcessor) => Promise<void>
> = new Map();

export function getSimpleProcessorRunner(
	src: string,
	dist: string,
	options: {
		env?: () => ITestEnv;
		context?: (src: string, dist: string) => ITestContext;
	} = {}
) {
	const createEnv =
		options.env || (() => ({ expect, it, beforeEach, afterEach }));
	const createContext =
		options.context ||
		((src: string, dist: string) => new TestContext({ src, dist }));
	const key = `src: ${src}, dist: ${dist}`;
	if (!CONTEXT_MAP.has(key)) {
		const context = createContext(src, dist);
		const runner = async function run(name: string, processor: ITestProcessor) {
			try {
				await processor.beforeAll?.(context);
				await processor.before?.(context);
				await processor.config?.(context);
				await processor.compiler?.(context);
				await processor.build?.(context);
			} catch (e: unknown) {
				context.emitError(name, e as Error);
			} finally {
				await processor.run?.(createEnv(), context);
				await processor.check?.(createEnv(), context);
				await processor.after?.(context);
				await processor.afterAll?.(context);
			}
		};
		CONTEXT_MAP.set(key, runner);
	}
	return CONTEXT_MAP.get(key)!;
}
