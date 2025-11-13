const fs = __non_webpack_require__("fs");
const path = __non_webpack_require__("path");

const buildAssetsPath = path.join(
	__dirname,
	"independent-share-build-assets.json"
);

const buildAssets = JSON.parse(fs.readFileSync(buildAssetsPath, "utf-8"));
const shareContainerPath = path.join(
	__dirname,
	"independent-share",
	"react",
	"react.container.js"
);


it("应该产出独立构建的 shared 资源", () => {
	const [fallbackName, fallbackFile] = buildAssets.react;
	const fallbackPath = path.join(__dirname, fallbackFile);
	expect(fs.existsSync(fallbackPath)).toBe(true);
	expect(fallbackName).toMatch(/^react/);
	const sharedStats = stats.shared.find(item => item.name === "react");
	expect(sharedStats.fallbackName).toBe(fallbackName);
	expect(sharedStats.fallback).toBe(fallbackFile);
	const sharedManifest = manifest.shared.find(item => item.name === "react");
	expect(sharedManifest.fallbackName).toBe(fallbackName);
	expect(sharedManifest.fallback).toBe(fallbackFile);
});

it("应该生成 share 容器文件并提供 get/init", () => {
	expect(fs.existsSync(shareContainerPath)).toBe(true);
	const module = require(shareContainerPath);
	expect(typeof module.get).toBe("function");
	expect(typeof module.init).toBe("function");
});
