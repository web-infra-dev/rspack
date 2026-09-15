import "./base.css";

function stylesheets(name) {
	return Array.from(document.getElementsByTagName("link")).filter(function (link) {
		return ((link.getAttribute("href") || "").split("?")[0]).endsWith(name);
	});
}

it("should load the stylesheet when a chunk gains css", async () => {
	await import(/* webpackChunkName: "lazy" */ "./lazy");
	if (typeof document !== "undefined") {
		expect(stylesheets("lazy.css").length).toBe(0);
	}
	await NEXT_HMR();
	if (typeof document !== "undefined") {
		const links = stylesheets("lazy.css");
		expect(links.length).toBe(1);
		expect(links[0].rel).toBe("stylesheet");
	}
});

module.hot.accept("./lazy");
