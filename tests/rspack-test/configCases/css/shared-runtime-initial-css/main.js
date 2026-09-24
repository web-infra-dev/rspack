it("should load a stylesheet that is initial only for another entry", async () => {
	const before = document.getElementsByTagName("link").length;
	const module = await import("./feature.js");
	expect(module.value).toBe(1);
	const links = Array.from(document.getElementsByTagName("link"));
	expect(links.length).toBe(before + 1);
	expect(links.some(link => link.href.includes("shared-css"))).toBe(true);
});
