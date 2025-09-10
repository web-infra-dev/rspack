it("nested new URL()", function () {
	const href1 = new URL("./a.txt", import.meta.url).href;
	const href2 = new URL(
		new URL("./a.txt", import.meta.url).href,
		import.meta.url
	).href;
	const href3 = new URL(
		new URL(new URL("./a.txt", import.meta.url).href, import.meta.url).href
	).href;
	expect(href1).toBe(href2);
	expect(href2).toBe(href3);
});
