const fs = require("fs");

require("./virtual-module-with-sourcemap");
require("./virtual-module-without-sourcemap");
require("./lib/components/virtual-component");

it("should extract source map from virtual modules", () => {
	const fileData = fs.readFileSync(__filename + ".map").toString("utf-8");
	const { sources } = JSON.parse(fileData);

	// Should include main file and virtual modules without inline sourcemap
	expect(sources).toContain("webpack:///./index.js");
	expect(sources).toContain("webpack:///./virtual-module-without-sourcemap.js");
	expect(sources).toContain("webpack:///./virtual-test.txt");
	expect(sources).toContain("webpack:///./src/components/virtual-component.js");

	// Should not include virtual modules with inline sourcemap as they're filtered by extractSourceMap
	expect(sources).not.toContain("webpack:///./virtual-module-with-sourcemap.js");
});

it("should handle virtual modules with extractSourceMap correctly", () => {
	const fileData = fs.readFileSync(__filename + ".map").toString("utf-8");
	const { sourcesContent } = JSON.parse(fileData);

	// Verify virtual module content is processed correctly
	const hasVirtualModuleContent = sourcesContent.some(content =>
		content && content.includes("virtual module content")
	);
	expect(hasVirtualModuleContent).toBe(true);

	// Verify extractSourceMap handles virtual modules correctly
	// Modules with inline sourcemap should be filtered, those without should be kept
	const hasModuleWithoutSourceMap = sourcesContent.some(content =>
		content && content.includes("const b = 2;")
	);
	expect(hasModuleWithoutSourceMap).toBe(true);
});

it("should handle virtual modules with different source map references", () => {
	const fileData = fs.readFileSync(__filename + ".map").toString("utf-8");
	const { sourcesContent } = JSON.parse(fileData);

	// Verify virtual modules with external .js.map references are processed correctly
	const hasMapReferenceContent = sourcesContent.some(content =>
		content && content.includes("module with map reference")
	);
	expect(hasMapReferenceContent).toBe(true);

	// Verify virtual modules with inline data URL source maps are processed correctly
	const hasAnotherModuleContent = sourcesContent.some(content =>
		content && content.includes("another module")
	);
	expect(hasAnotherModuleContent).toBe(true);

	// Note: extractSourceMap extracts inline source maps but may not remove all sourceMappingURL comments
	// The key behavior is that modules with inline source maps are filtered from the source map sources
});