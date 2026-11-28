function stylesheets() {
	return Array.from(document.getElementsByTagName("link"))
		.filter(link => link.rel === "stylesheet")
		.map(link => link.getAttribute("href"));
}

it("should load each stylesheet when the public path has a query", async () => {
	await import(/* webpackChunkName: "a" */ "./a");
	expect(stylesheets()).toEqual(["https://test.cases/path/asset?file=a.css"]);
	await import(/* webpackChunkName: "b" */ "./b");
	expect(stylesheets()).toEqual([
		"https://test.cases/path/asset?file=a.css",
		"https://test.cases/path/asset?file=b.css"
	]);
});
