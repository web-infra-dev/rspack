import value from "./app";

function stylesheets() {
	return Array.from(document.getElementsByTagName("link")).filter(link =>
		(link.getAttribute("href") || "").split("?")[0].endsWith("main.css")
	);
}

async function waitForReload(oldTag) {
	if (typeof document === "undefined") return;
	const deadline = Date.now() + 5000;
	while (stylesheets()[2] === oldTag && Date.now() < deadline) {
		await new Promise(resolve => setTimeout(resolve, 10));
	}
	expect(stylesheets()[2]).not.toBe(oldTag);
}

it("should preserve framework-owned stylesheets across updates and removal", async () => {
	expect(value).toBe(1);
	let original;
	let foreign;
	let owned;
	if (typeof document !== "undefined") {
		[original] = stylesheets();
		expect(original.getAttribute("data-rspack")).toBe(null);
		foreign = original.cloneNode();
		foreign.setAttribute("data-rspack", "another-build:mini-css-chunk-main");
		document.head.appendChild(foreign);
		owned = original.cloneNode();
		owned.setAttribute("data-rspack", "css-ownership:css");
		document.head.appendChild(owned);
	}
	await NEXT_HMR();
	// Wait for the debounced reload and the new stylesheet load event.
	await waitForReload(owned);
	expect(require("./app").default).toBe(2);
	if (typeof document !== "undefined") {
		const links = stylesheets();
		expect(links).toHaveLength(3);
		expect(links.slice(0, 2)).toEqual([original, foreign]);
		expect(links[2]).not.toBe(owned);
		expect(links[2].getAttribute("data-rspack")).toBe("css-ownership:css");
		owned = links[2];
	}
	await NEXT_HMR();
	await waitForReload(owned);
	expect(require("./app").default).toBe(3);
	if (typeof document !== "undefined") {
		expect(stylesheets().slice(0, 2)).toEqual([original, foreign]);
		document.head.removeChild(stylesheets()[2]);
		// Simulate the owner unmounting its original nodes after HMR.
		document.head.removeChild(original);
		document.head.removeChild(foreign);
		expect(stylesheets()).toEqual([]);
	}
});

module.hot.accept("./app");
