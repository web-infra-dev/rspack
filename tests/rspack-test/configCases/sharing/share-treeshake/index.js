const fs = __non_webpack_require__("fs");
const path = __non_webpack_require__("path");

const statsPath = path.join(__dirname, "mf-stats.json");
const manifestPath = path.join(__dirname, "mf-manifest.json");
const stats = JSON.parse(fs.readFileSync(statsPath, "utf-8"));
const manifest = JSON.parse(fs.readFileSync(manifestPath, "utf-8"));

it("应该仅为共享依赖生成一个产物", () => {
	const remoteEntryFile = stats.metaData.remoteEntry.name;
	const remoteEntryPath = path.join(__dirname, remoteEntryFile);
	expect(fs.existsSync(remoteEntryPath)).toBe(true);
	expect(stats.shared).toHaveLength(1);
});

it("应该记录共享依赖被实际使用的导出", () => {
	const sharedStats = stats.shared[0];
	const sharedManifest = manifest.shared.find(item => item.name === "react");
	expect(sharedStats.usedExports).toEqual(["default"]);
	expect(sharedManifest.usedExports).toEqual(["default"]);
});

it("应该生成共享依赖的独立 fallback", () => {
	const buildAssetsPath = path.join(
		__dirname,
		"independent-share-build-assets.json"
	);
	const buildAssets = JSON.parse(fs.readFileSync(buildAssetsPath, "utf-8"));
	expect(buildAssets.react).toBeDefined();
	const [, fallbackFile] = buildAssets.react;
	expect(stats.shared[0].fallback).toBe(fallbackFile);
	expect(manifest.shared.find(item => item.name === "react").fallback).toBe(
		fallbackFile
	);
});
