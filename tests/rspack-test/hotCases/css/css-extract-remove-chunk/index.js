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
	await NEXT_HMR();
	if (typeof document !== "undefined") {
		expect(stylesheets().length).toBe(0);
	}
});

module.hot.accept("./module");
