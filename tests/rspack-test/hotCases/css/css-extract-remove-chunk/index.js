import getChunk from "./module";

function stylesheets() {
	return Array.from(document.getElementsByTagName("link")).filter(function (link) {
		return ((link.getAttribute("href") || "").split("?")[0]).endsWith("lazy.css");
	});
}

it("should remove the stylesheet when the chunk is removed", async () => {
	await getChunk();
	let external;
	if (typeof document !== "undefined") {
		expect(stylesheets().length).toBe(1);
		const owned = stylesheets()[0];
		expect(owned.getAttribute("data-rspack")).toBe("css-owned:mini-css-chunk-lazy");
		external = owned.cloneNode();
		external.removeAttribute("data-rspack");
		document.head.insertBefore(external, owned);
	}
	await NEXT_HMR();
	if (typeof document !== "undefined") {
		expect(stylesheets()).toEqual([external]);
		document.head.removeChild(external);
		expect(stylesheets().length).toBe(0);
	}
});

module.hot.accept("./module");
