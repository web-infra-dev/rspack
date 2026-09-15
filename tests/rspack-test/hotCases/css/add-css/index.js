import "./base.css";

function stylesheets() {
	return Array.from(document.getElementsByTagName("link")).filter(function (link) {
		return ((link.getAttribute("href") || "").split("?")[0]).endsWith("lazy.css");
	});
}

it("should load the stylesheet when a chunk gains css", async () => {
	await import(/* webpackChunkName: "lazy" */ "./lazy");
	const hasCssHmr =
		typeof document !== "undefined" && !!__webpack_require__.hmrC.css;
	if (hasCssHmr) {
		expect(stylesheets().length).toBe(0);
	}
	await NEXT_HMR();
	if (hasCssHmr) {
		expect(stylesheets().length).toBe(1);
	}
});

module.hot.accept("./lazy");
