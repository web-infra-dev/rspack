it("should not affect contexts that don't match the regex", function () {
	// This context does NOT match /components$/ so should not be affected
	var assetsContext = require.context("./assets", false, /\.js$/);
	var assetKeys = assetsContext.keys();

	// Should find the asset files since the plugin shouldn't affect this context
	expect(assetKeys.length).toBe(2);
	expect(assetKeys).toContain("./file1.js");
	expect(assetKeys).toContain("./file2.js");

	// Verify we can actually load the files
	expect(assetsContext("./file1.js")).toBe("asset1");
	expect(assetsContext("./file2.js")).toBe("asset2");
});
