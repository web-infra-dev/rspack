it("should generate expected manifest", async () => {
	const fs = __non_webpack_require__("fs");
	const path = __non_webpack_require__("path");
	const statsPath = path.join(__dirname, "mf-stats.json");
	const manifestPath = path.join(__dirname, "mf-manifest.json");

	const stats = JSON.parse(fs.readFileSync(statsPath, "utf-8"));
	const manifest = JSON.parse(fs.readFileSync(manifestPath, "utf-8"));
	const remoteEntryFile = stats.metaData.remoteEntry.name;
	const remoteEntryPath = path.join(__dirname, remoteEntryFile);
	expect(fs.existsSync(remoteEntryPath)).toBe(true);

	expect(stats.shared).toHaveLength(1);
	expect(stats.shared[0].assets.js.sync.sort()).toEqual([
		"lazy-module_js.js",
		"node_modules_react_js.js"
	]);
	expect(stats.shared[0].assets.js.async).toEqual([]);

	expect(stats.exposes).toHaveLength(1);
	expect(stats.exposes[0].assets.js.sync).toEqual(["module_js.js"]);
	expect(stats.exposes[0].assets.js.async).toEqual([]);

	expect(manifest.shared).toEqual(
		expect.arrayContaining([
			expect.objectContaining({
				name: "react",
				assets: expect.objectContaining({
					js: expect.objectContaining({
						sync: expect.arrayContaining([
							"lazy-module_js.js",
							"node_modules_react_js.js"
						]),
						async: []
					})
				})
			})
		])
	);
	expect(manifest.exposes).toEqual(
		expect.arrayContaining([
			expect.objectContaining({
				name: "expose-a",
				assets: expect.objectContaining({
					js: expect.objectContaining({
						sync: ["module_js.js"],
						async: []
					})
				})
			})
		])
	);
	expect(manifest.extra).toBe(true);
});
