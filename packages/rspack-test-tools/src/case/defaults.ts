import path from "path";

import {
	DefaultsConfigProcessor,
	IDefaultsConfigProcessorOptions
} from "../processor";
import { TestContext } from "../test/context";
import { ECompilerType, ITestProcessor } from "../type";

export type TDefaultsCaseConfig = Omit<
	IDefaultsConfigProcessorOptions<ECompilerType.Rspack>,
	"name" | "compilerType"
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
		await processor.check?.({ expect, it, beforeEach, afterEach, fn: jest.fn, spyOn: jest.spyOn }, context);
		await processor.after?.(context);
	}
}

export function createDefaultsCase(name: string, src: string) {
	const caseConfig = require(src) as TDefaultsCaseConfig;
	it(`should generate the correct defaults from ${caseConfig.description}`, async () => {
		await run(
			name,
			new DefaultsConfigProcessor({
				name,
				compilerType: ECompilerType.Rspack,
				...caseConfig
			})
		);
	});
}
