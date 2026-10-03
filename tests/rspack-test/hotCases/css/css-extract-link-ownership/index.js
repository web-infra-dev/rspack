import value from "./app";

function stylesheets() {
	return Array.from(document.getElementsByTagName("link")).filter(link =>
		(link.getAttribute("href") || "").split("?")[0].endsWith("main.css")
	);
}

it("should preserve framework-owned stylesheets across updates and removal", async () => {
	expect(value).toBe(1);
	let original;
	let foreign;
	if (typeof document !== "undefined") {
		[original] = stylesheets();
		expect(original.getAttribute("data-rspack")).toBe(null);
		foreign = original.cloneNode();
		foreign.setAttribute("data-rspack", "another-build:mini-css-chunk-main");
		document.head.appendChild(foreign);
	}
	await NEXT_HMR();
	// The loader's fallback uses a 50ms debounce.
	await new Promise(resolve => setTimeout(resolve, 100));
	expect(require("./app").default).toBe(2);
	if (typeof document !== "undefined") {
		expect(stylesheets()).toEqual([original, foreign]);
	}
	await NEXT_HMR();
	await new Promise(resolve => setTimeout(resolve, 100));
	expect(require("./app").default).toBe(3);
	if (typeof document !== "undefined") {
		expect(stylesheets()).toEqual([original, foreign]);
		// Simulate the owner unmounting its original nodes after HMR.
		document.head.removeChild(original);
		document.head.removeChild(foreign);
		expect(stylesheets()).toEqual([]);
	}
});

module.hot.accept("./app");
