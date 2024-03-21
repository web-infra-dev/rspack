import { TestContext } from "../test/context";
import path from "path";
import { ITestEnv, ITestProcessor } from "../type";
import { DefaultsConfigTaskProcessor } from "../processor";

const srcDir = path.resolve(__dirname, "../../../rspack/tests/fixtures");
const distDir = path.resolve(__dirname, "../../../rspack/tests/js/compiler");

const context = new TestContext({
	src: srcDir,
	dist: distDir
});

async function run(name: string, processor: ITestProcessor) {
	try {
		await processor.before?.(context);
		await processor.config?.(context);
	} catch (e: unknown) {
		context.emitError(name, e as Error);
	} finally {
		await processor.check?.(null as unknown as ITestEnv, context);
		await processor.after?.(context);
	}
}

export function createDefaultsCase(src: string) {
	const caseConfig = require(src);
	it(`should generate the correct defaults from ${caseConfig.description}`, async () => {
		await run(caseConfig.name, new DefaultsConfigTaskProcessor(caseConfig));
	});
}
