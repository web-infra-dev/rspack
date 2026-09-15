const jsUrlA = new URL("./target-a.js", import.meta.url);
const jsUrlB = new URL("./target-b.js", import.meta.url);
const cssUrl = new URL("./target.css", import.meta.url);
const cssModuleUrl = new URL("./target-module.css", import.meta.url);
const cssTextUrl = new URL("./target-export.css?text", import.meta.url);
const cssStyleUrl = new URL("./target-export.css?style", import.meta.url);
const cssStyleSheetUrl = new URL(
	"./target-export.css?css-style-sheet",
	import.meta.url
);
const assetUrl = new URL("./target.png", import.meta.url);
const jsAssetUrl = new URL("./target-asset.js", import.meta.url);
const cssAssetUrl = new URL("./target-asset.css", import.meta.url);

it("should turn JavaScript and CSS URL dependencies into async entries", () => {
	const scriptExtension = URL_MODE === "new-url-relative" ? "mjs" : "js";
	const urlPrefix =
		URL_MODE === "relative"
			? "assets/"
			: URL_MODE === "new-url-relative"
				? "/"
				: "/assets/";
	const scriptUrlPattern = new RegExp(
		`${urlPrefix}url-${URL_MODE}-[^/]+\\.${scriptExtension}$`
	);
	const cssUrlPattern = new RegExp(
		`${urlPrefix}url-${URL_MODE}-[^/]+\\.css$`
	);

	expect(jsUrlA.href).toMatch(scriptUrlPattern);
	expect(jsUrlB.href).toMatch(scriptUrlPattern);
	expect(jsUrlA.href).not.toBe(jsUrlB.href);
	expect(cssUrl.href).toMatch(cssUrlPattern);
	expect(cssModuleUrl.href).toMatch(scriptUrlPattern);
	for (const url of [cssTextUrl, cssStyleUrl, cssStyleSheetUrl]) {
		expect(url.href).toMatch(scriptUrlPattern);
	}
	expect(assetUrl.href).toMatch(
		new RegExp(`${urlPrefix}target-${URL_MODE}\\.png$`)
	);
	expect(jsAssetUrl.href).toMatch(
		new RegExp(`${urlPrefix}target-asset-${URL_MODE}\\.js$`)
	);
	expect(cssAssetUrl.href).toMatch(
		new RegExp(`${urlPrefix}target-asset-${URL_MODE}\\.css$`)
	);

	expect(globalThis.URL_ENTRY_TARGET_A_EXECUTED).toBeUndefined();
	expect(globalThis.URL_ENTRY_TARGET_B_EXECUTED).toBeUndefined();
});
