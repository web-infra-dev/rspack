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
	const original = typeof document !== "undefined" && stylesheets()[0];
	await NEXT_HMR();
	if (typeof document !== "undefined") {
		expect(stylesheets()).toEqual([original]);
		expect(original.disabled).toBe(true);
	}
});

module.hot.accept("./app");
