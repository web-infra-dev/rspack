import value from "./app";

function stylesheets() {
	return Array.from(document.getElementsByTagName("link")).filter(function (link) {
		return ((link.getAttribute("href") || "").split("?")[0]).endsWith("main.css");
	});
}

it("should remove the stylesheet when the chunk loses its css", async () => {
	expect(value).toBe(1);
	if (typeof document !== "undefined") {
		expect(stylesheets().length).toBe(1);
	}
	await NEXT_HMR();
	if (typeof document !== "undefined") {
		expect(stylesheets().length).toBe(0);
	}
});

module.hot.accept("./app");
