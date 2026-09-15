it("should import an external css", async () => {
	const x = await import("./style.css");
	expect(x).toEqual(nsObj({}));

	const style = getComputedStyle(document.body);
	expect(style.getPropertyValue("background")).toBe(
		"url(\"//example.com/image.png\")"
	);
	const getCss = () => Object.values(window["__LINK_SHEET__"]).join("\n");
	expect(getCss()).toContain('background-image: url("http://example.com/image.png")');
	await new Promise(resolve => setTimeout(resolve, 200));
	expect(getCss()).toContain("color: green");
});
