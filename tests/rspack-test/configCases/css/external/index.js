it("should import an external css", async () => {
	const x = await import("./style.css");
	expect(x).toEqual(nsObj({}));

	let style = getComputedStyle(document.body);
	expect(style.getPropertyValue("background")).toBe(
		"url(\"//example.com/image.png\")"
	);
	expect(style.getPropertyValue("background-image")).toBe(
		"url(\"http://example.com/image.png\")"
	);
	await new Promise(resolve => setTimeout(resolve, 200));
	style = getComputedStyle(document.body);
	expect(style.getPropertyValue("color")).toBe("rgb(0, 128, 0)");
});
