const fs = __non_webpack_require__("fs");
const path = __non_webpack_require__("path");

const statsPath = path.join(__dirname, "mf-stats.json");
const manifestPath = path.join(__dirname, "mf-manifest.json");
const stats = JSON.parse(fs.readFileSync(statsPath, "utf-8"));
const manifest = JSON.parse(fs.readFileSync(manifestPath, "utf-8"));

it("should emit remote entry with hash", () => {
	const remoteEntryFile = stats.metaData.remoteEntry.name;
	const remoteEntryPath = path.join(__dirname, remoteEntryFile);
	expect(fs.existsSync(remoteEntryPath)).toBe(true);
});

// shared
it("should report shared assets in sync only", () => {
	expect(stats.shared).toHaveLength(1);
	expect(stats.shared[0].assets.js.sync.sort()).toEqual([
		"node_modules_react_js.js"
	]);
	expect(stats.shared[0].assets.js.async).toEqual([]);
});

it("should materialize in manifest", () => {
	expect(manifest.shared).toEqual(
		expect.arrayContaining([
			expect.objectContaining({
				name: "react",
				assets: expect.objectContaining({
					js: expect.objectContaining({
						sync: expect.arrayContaining([
							"node_modules_react_js.js"
						]),
						async: []
					})
				})
			})
		])
	);
});

//exposes
it("should expose sync assets only", () => {
	expect(stats.exposes).toHaveLength(1);
	expect(stats.exposes[0].assets.js.sync).toEqual(["_federation_expose_a.js"]);
	expect(stats.exposes[0].assets.js.async).toEqual([
		"lazy-module_js.js"
	]);
});

it("should reflect expose assets in manifest", () => {
	expect(manifest.exposes).toEqual(
		expect.arrayContaining([
			expect.objectContaining({
				name: "expose-a",
				path: "./expose-a",
				assets: expect.objectContaining({
					js: expect.objectContaining({
						sync: ["_federation_expose_a.js"],
						async: [
							"lazy-module_js.js",
						]
					})
				})
			})
		])
	);
});

// remotes

it("should record remote usage", () => {
	expect(stats.remotes).toEqual(
		expect.arrayContaining([
			expect.objectContaining({
				alias: "@remote/alias",
				consumingFederationContainerName: "container",
				federationContainerName: "remote",
				moduleName: ".",
				usedIn: expect.arrayContaining([
					"module.js"
				]),
				entry: 'http://localhost:8000/remoteEntry.js'
			})
		])
	);
});

it("should persist remote metadata in manifest", () => {
	expect(manifest.remotes).toEqual(
		expect.arrayContaining([
			expect.objectContaining({
				alias: "@remote/alias",
				federationContainerName: "remote",
				moduleName: "."
			})
		])
	);
});
