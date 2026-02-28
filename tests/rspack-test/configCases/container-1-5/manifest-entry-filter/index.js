
const fs = __non_webpack_require__("fs");
const path = __non_webpack_require__("path");

const statsPath = path.join(__dirname, "mf-stats.json");
const stats = JSON.parse(fs.readFileSync(statsPath, "utf-8"));

//exposes
it("should expose sync assets only and filter out host specific chunks", () => {
	expect(stats.exposes).toHaveLength(1);
	expect(stats.exposes[0].file).toBe("module.js");
	expect(stats.exposes[0].assets.js.sync).toEqual(["_federation_expose_a.js"]);
	expect(stats.exposes[0].assets.js.async).toEqual([
		"lazy-module_js.js"
	]);
	// Ensure host app's specific chunks are NOT present
	expect(stats.exposes[0].assets.js.sync).not.toContain("host.js");
	expect(stats.exposes[0].assets.js.sync).not.toContain("vendors-main-vendor.js");
	expect(stats.exposes[0].assets.js.async).not.toContain("async-main.js");
});

it("should report xreact shared assets in sync only", () => {
    const xreact = stats.shared.find(item => item.name === "xreact");
    expect(xreact).toBeDefined();
    // Shared module should still be present
    expect(xreact.assets.js.sync).toEqual(["node_modules_xreact_index_js.js"]);
});
