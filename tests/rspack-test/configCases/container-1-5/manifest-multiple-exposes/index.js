const fs = __non_webpack_require__("fs");
const path = __non_webpack_require__("path");

const stats = JSON.parse(
	fs.readFileSync(path.join(__dirname, "mf-stats.json"), "utf-8")
);
const manifest = JSON.parse(
	fs.readFileSync(path.join(__dirname, "mf-manifest.json"), "utf-8")
);

function getExpose(source, name) {
	return source.exposes.find(item => item.name === name);
}

it("should keep expose entry assets separated in stats", () => {
	const exposeA = getExpose(stats, "expose-a");
	const exposeB = getExpose(stats, "expose-b");

	expect(exposeA.assets.js.sync).toContain("__federation_expose_expose-a.js");
	expect(exposeA.assets.js.sync).not.toContain("__federation_expose_expose-b.js");
	expect(exposeB.assets.js.sync).toContain("__federation_expose_expose-b.js");
	expect(exposeB.assets.js.sync).not.toContain("__federation_expose_expose-a.js");
});

it("should keep expose entry assets separated in manifest", () => {
	const exposeA = getExpose(manifest, "expose-a");
	const exposeB = getExpose(manifest, "expose-b");

	expect(exposeA.assets.js.sync).toContain("__federation_expose_expose-a.js");
	expect(exposeA.assets.js.sync).not.toContain("__federation_expose_expose-b.js");
	expect(exposeB.assets.js.sync).toContain("__federation_expose_expose-b.js");
	expect(exposeB.assets.js.sync).not.toContain("__federation_expose_expose-a.js");
});
