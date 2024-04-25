const path = require("path");
const fs = require("fs");
const srcDir = __dirname;
const caseDir = path.resolve(__dirname, "./statsAPICases");
const { StatsAPITaskProcessor, TestContext, ECompilerType } = require("..");

describe("Stats", () => {
	StatsAPITaskProcessor.addSnapshotSerializer();
	const context = new TestContext({
		src: srcDir,
		dist: "none"
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
				new StatsAPITaskProcessor({
					name: file,
					compilerType: ECompilerType.Rspack,
					...caseConfig
				})
			);
		});
	}
});
