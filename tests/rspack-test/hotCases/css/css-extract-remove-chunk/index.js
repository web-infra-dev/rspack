import getChunk from "./module";

function stylesheets() {
	return Array.from(document.getElementsByTagName("link")).filter(function (link) {
		return ((link.getAttribute("href") || "").split("?")[0]).endsWith("lazy.css");
	});
}

it("should keep a removed chunk's stylesheet disabled until the chunk is loaded again", async () => {
	await getChunk();
	if (typeof document !== "undefined") {
		expect(stylesheets().length).toBe(1);
	}
	const original = typeof document !== "undefined" && stylesheets()[0];
	await NEXT_HMR();
	if (typeof document !== "undefined") {
		expect(stylesheets()).toEqual([original]);
		expect(original.disabled).toBe(true);
	}
	await NEXT_HMR();
	if (typeof document !== "undefined") {
		expect(stylesheets()).toEqual([original]);
		expect(original.disabled).toBe(true);
	}
	const disabledHref = typeof document !== "undefined" && original.href;
	await getChunk();
	if (typeof document !== "undefined") {
		expect(stylesheets()).toEqual([original]);
		expect(original.disabled).toBe(false);
		expect(original.href).not.toBe(disabledHref);
	}
});

module.hot.accept("./module");
