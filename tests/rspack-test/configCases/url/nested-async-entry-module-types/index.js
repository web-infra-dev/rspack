it("should promote JavaScript URLs inside nested async blocks without executing them", async () => {
	const [jsUrlA, jsUrlB, assetUrl, jsAssetUrl] = await new Promise((resolve, reject) => {
		require.ensure([], require => {
			const jsUrlA = new URL("./target-a.js", import.meta.url);
			require.ensure([], require => {
				resolve([
					jsUrlA,
					new URL("./target-b.js", import.meta.url),
					new URL("./target.png", import.meta.url),
					new URL("./target-asset.js", import.meta.url),
				]);
			}, reject, "inner");
		}, reject, "outer");
	});
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

	expect(jsUrlA.href).toMatch(scriptUrlPattern);
	expect(jsUrlB.href).toMatch(scriptUrlPattern);
	expect(jsUrlA.href).not.toBe(jsUrlB.href);
	expect(assetUrl.href).toMatch(
		new RegExp(`${urlPrefix}target-${URL_MODE}\\.png$`)
	);
	expect(jsAssetUrl.href).toMatch(
		new RegExp(`${urlPrefix}target-asset-${URL_MODE}\\.js$`)
	);

	expect(globalThis.URL_ENTRY_TARGET_A_EXECUTED).toBeUndefined();
	expect(globalThis.URL_ENTRY_TARGET_B_EXECUTED).toBeUndefined();
	expect(globalThis.URL_ENTRY_JS_ASSET_EXECUTED).toBeUndefined();
});
