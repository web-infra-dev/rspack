it(`should load another entry's initial CSS with ${runtimeChunkMode} runtime`, async () => {
	const initialLinks = Array.from(document.getElementsByTagName("link"));
	expect(initialLinks.length).toBe(preloadedCss ? 1 : 0);
	expect(getCssRequests()).toHaveLength(preloadedCss ? 1 : 0);
	if (preloadedCss) expect(initialLinks[0].sheet).toBeTruthy();

	const appendChild = document.head.appendChild;
	const stylesheets = [];
	let resolveScriptLoaded;
	const scriptLoaded = new Promise(resolve => {
		resolveScriptLoaded = resolve;
	});
	let settled = false;
	let feature;

	document.head.appendChild = function (element) {
		if (element.tagName === "LINK" && element.rel === "stylesheet") {
			// Hold CSS until after the JavaScript has loaded, so an import that
			// only waits for JavaScript cannot accidentally pass the test.
			stylesheets.push(element);
			return element;
		}
		if (element.tagName === "SCRIPT") {
			element.addEventListener("load", resolveScriptLoaded, { once: true });
		}
		return appendChild.call(this, element);
	};

	try {
		feature = import("./feature.js").then(module => {
			settled = true;
			return module;
		});
		await Promise.race([scriptLoaded, feature]);
		await new Promise(resolve => setTimeout(resolve, 0));

		expect({ stylesheets: stylesheets.map(link => link.href), settled }).toEqual({
			stylesheets: preloadedCss
				? []
				: [`https://test.cases/path/shared-css.${runtimeChunkMode}.css`],
			settled: preloadedCss
		});
	} finally {
		document.head.appendChild = appendChild;
		for (const stylesheet of stylesheets) {
			appendChild.call(document.head, stylesheet);
		}
		if (feature) await feature;
	}

	expect((await feature).value).toBe(42);
	expect(settled).toBe(true);
	expect(document.getElementsByTagName("link").length).toBe(1);
	if (preloadedCss) {
		expect(document.getElementsByTagName("link")[0]).toBe(initialLinks[0]);
	}

	await import("./feature.js");
	expect(getCssRequests()).toEqual([
		`https://test.cases/path/shared-css.${runtimeChunkMode}.css`
	]);
	expect(document.getElementsByTagName("link").length).toBe(1);
});
