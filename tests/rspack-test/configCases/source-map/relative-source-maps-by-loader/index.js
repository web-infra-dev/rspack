it("should run all inline loaders without a resource", () => {
	expect(require("./loader-source-root.mjs!")).toBe("ok");
	expect(require("./loader-source-root-slash.mjs!")).toBe("ok");
	expect(require("./loader-source-root-source-slash.mjs!")).toBe("ok");
	expect(require("./loader-source-root-2-slash.mjs!")).toBe("ok");
	expect(require("./loader-no-source-root.mjs!")).toBe("ok");
	expect(require("./loader-pre-relative.mjs!")).toBe("ok");
});

it("should generate the correct SourceMap", function () {
	const fs = require("fs");
	const path = require("path");
	const source = JSON.parse(fs.readFileSync(__filename + ".map", "utf-8"));
	let sourceUrl = sourceName => `webpack:///${sourceName}`;
	if (globalThis.__RSPACK_TEST_RUNTIME_MODE_RSPACK) {
		sourceUrl = sourceName => `rspack:///${sourceName}`;
	}
	expect(source.sources).toContain(sourceUrl("./folder/test1.txt"));
	expect(source.sources).toContain(sourceUrl("./folder/test3.txt"));
	expect(source.sources).toContain(sourceUrl("./folder/test5.txt"));
	expect(source.sources).toContain(sourceUrl("./folder/test6.txt"));
	// Absolute sources override sourceRoot, with or without its trailing slash.
	for (const name of ["test2.txt", "test4.txt"]) {
		const relativePath = path.relative(TEST_CONTEXT, `/${name}`).replace(/\\/g, "/");
		expect(source.sources).toContain(sourceUrl(relativePath));
		expect(source.sources).not.toContain(sourceUrl(`./folder/${name}`));
	}
});
