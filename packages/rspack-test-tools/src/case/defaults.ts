import path from "path";

import {
	DefaultsConfigTaskProcessor,
	IDefaultsConfigProcessorOptions
} from "../processor";
import { TestContext } from "../test/context";
import { ITestEnv, ITestProcessor } from "../type";

export type TDefaultsCaseConfig = Omit<
	IDefaultsConfigProcessorOptions,
	"name"
> & {
	description: string;
};

const srcDir = path.resolve(__dirname, "../../tests/fixtures");
const distDir = path.resolve(__dirname, "../../tests/js/defaults");

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

export function createDefaultsCase(name: string, src: string) {
	const caseConfig = require(src) as TDefaultsCaseConfig;
	it(`should generate the correct defaults from ${caseConfig.description}`, async () => {
		await run(
			name,
			new DefaultsConfigTaskProcessor({
				name,
				...caseConfig
			})
		);
	});
}
