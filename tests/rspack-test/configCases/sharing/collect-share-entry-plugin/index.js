const fs = __non_webpack_require__("fs");
const path = __non_webpack_require__("path");

it("should emit collect share entry asset with expected requests", async () => {
	await import('./module');
	const assetPath = path.join(__dirname, "collect-share-entries.json");
	expect(fs.existsSync(assetPath)).toBe(true);

	const content = JSON.parse(fs.readFileSync(assetPath, "utf-8"));
	expect(content.shared).toBeDefined();

	const collectInfo = content.shared["xreact"];
	expect(collectInfo).toBeDefined();
	expect(collectInfo.shareScope).toBe("default");
	expect(collectInfo.requests[0][0]).toContain("sharing/collect-share-entry-plugin/node_modules/xreact/index.js");
	expect(collectInfo.requests[0][1]).toEqual("1.0.0");
});
