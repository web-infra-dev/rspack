const fs = require("fs");
const path = require("path");
const { SourceMapConsumer } = require("source-map");

it("should map the module loader to its bootstrap source", async () => {
	expect(require("./dep")).toBe(42);
	// Using the module cache also exercises startup without entry inlining.
	expect(require.cache).toBeDefined();
	const filename = path.join(__dirname, RUNTIME_FILENAME);
	const code = fs.readFileSync(filename, "utf8");
	const map = JSON.parse(fs.readFileSync(filename + ".map", "utf8"));
	const prefix = globalThis.__RSPACK_TEST_RUNTIME_MODE_RSPACK ? "rspack" : "webpack";
	const bootstrapSource = `${prefix}:///${prefix}/bootstrap`;
	const sourceIndex = map.sources.indexOf(bootstrapSource);
	expect(sourceIndex).toBeGreaterThanOrEqual(0);
	const bootstrap = map.sourcesContent[sourceIndex];
	expect(bootstrap).toContain("// The module cache");
	const requireName = prefix === "rspack" ? "__rspack_require" : "__webpack_require__";

	const consumer = await new SourceMapConsumer(map);
	try {
		// Locate statements in the original bootstrap so this works even when
		// the minifier renames the loader and combines its statements.
		for (const statement of [
			`function ${requireName}(moduleId)`,
			"var cachedModule =",
			"if (cachedModule !== undefined)",
			"return cachedModule.exports;",
			"return module.exports;",
		]) {
			const offset = bootstrap.indexOf(statement);
			expect(offset).toBeGreaterThanOrEqual(0);
			const lines = bootstrap.slice(0, offset).split("\n");
			const generated = consumer.generatedPositionFor({
				source: bootstrapSource,
				line: lines.length,
				column: lines[lines.length - 1].length,
				bias: SourceMapConsumer.LEAST_UPPER_BOUND,
			});
			expect(generated.line).not.toBeNull();
			const original = consumer.originalPositionFor(generated);
			expect(original.source).toBe(bootstrapSource);
			expect(original.line).toBe(lines.length);
			expect(code.split("\n")[generated.line - 1].length).toBeGreaterThan(generated.column);
		}
		if (EXPECT_STARTUP) {
			expect(map.sources).toContain(`${prefix}:///${prefix}/startup`);
		}
	} finally {
		consumer.destroy();
	}
});
