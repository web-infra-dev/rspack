import getChunk from "./module";

function stylesheets() {
	return Array.from(document.getElementsByTagName("link")).filter(function (link) {
		return ((link.getAttribute("href") || "").split("?")[0]).endsWith("lazy.css");
	});
}

it("should remove the stylesheet when the chunk is removed", async () => {
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
});

module.hot.accept("./module");
