import "./base.css";

function stylesheets() {
	return Array.from(document.getElementsByTagName("link")).filter(function (link) {
		return /\/(constructor|toString)\.css(?:\?|$)/.test(link.href);
	});
}

it("loads the first stylesheet of loaded chunks named after Object.prototype properties", async () => {
	const constructorChunk = await import(/* webpackChunkName: "constructor" */ "./constructor");
	const toStringChunk = await import(/* webpackChunkName: "toString" */ "./to-string");
	expect(constructorChunk.default).toBe(1);
	expect(toStringChunk.default).toBe(1);
	expect(stylesheets()).toHaveLength(0);

	await NEXT_HMR();

	const links = stylesheets().sort((a, b) => a.href.localeCompare(b.href));
	expect(links.map(link => link.getAttribute("href"))).toEqual([
		expect.stringContaining("constructor.css?hmr="),
		expect.stringContaining("toString.css?hmr=")
	]);
	for (const link of links) {
		expect(getLinkSheet(link)).toContain("color: blue");
	}
});

module.hot.accept(["./constructor", "./to-string"]);
