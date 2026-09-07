import value from "./app";

function stylesheets() {
	return Array.from(document.getElementsByTagName("link")).filter(function (link) {
		return ((link.getAttribute("href") || "").split("?")[0]).endsWith("main.css");
	});
}

it("should re-apply the stylesheet when the css import order changes", async () => {
	expect(value).toBe(1);
	if (typeof document !== "undefined") {
		const links = stylesheets();
		expect(links.length).toBe(1);
		expect((links[0].getAttribute("href") || "").indexOf("?")).toBe(-1);
	}
	await NEXT_HMR();
	if (typeof document !== "undefined") {
		const links = stylesheets();
		expect(links.length).toBe(1);
		// The cascade order changed, so the stylesheet must have been re-fetched
		// (the swapped-in probe carries a cache-busting query).
		expect((links[0].getAttribute("href") || "").indexOf("?")).not.toBe(-1);
	}
});

module.hot.accept("./app");
