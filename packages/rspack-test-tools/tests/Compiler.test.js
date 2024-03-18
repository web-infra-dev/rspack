const path = require("path");
const fs = require("fs");
const srcDir = path.resolve(__dirname, "../../rspack/tests/fixtures");
const distDir = path.resolve(__dirname, "../../rspack/tests/js/compiler");
const caseDir = path.resolve(__dirname, "./compilerCases");
const { SimpleTaskProcessor, TestContext, ECompilerType } = require("..");

describe("Compiler", () => {
	const context = new TestContext({
		src: srcDir,
		dist: distDir
	});

	async function run(name, processor) {
		try {
			await processor.before(context);
			await processor.config(context);
			await processor.compiler(context);
			await processor.build(context);
		} catch (e) {
			context.emitError(name, e);
		} finally {
			await processor.check(null, context);
			await processor.after(context);
		}
	}

	const cases = fs.readdirSync(caseDir);
	for (let file of cases) {
		const caseConfig = require(path.join(caseDir, file));
		it(caseConfig.description, async () => {
			await run(
				file,
				new SimpleTaskProcessor({
					name: file,
					compilerType: ECompilerType.Rspack,
					...caseConfig
				})
			);
		});
	}
});
