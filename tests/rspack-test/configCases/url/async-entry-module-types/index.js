const jsUrlA = new URL("./target-a.js", import.meta.url);
const jsUrlB = new URL("./target-b.js", import.meta.url);
const cssUrl = new URL("./target.css", import.meta.url);
const assetUrl = new URL("./target.png", import.meta.url);

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
	expect(assetUrl.href).toMatch(
		new RegExp(`${urlPrefix}target-${URL_MODE}\\.png$`)
	);

	expect(globalThis.URL_ENTRY_TARGET_A_EXECUTED).toBeUndefined();
	expect(globalThis.URL_ENTRY_TARGET_B_EXECUTED).toBeUndefined();
});
