it("should import an external css", async () => {
	const x = await import("./style.css");
	expect(x).toEqual(nsObj({}));

	const style = getComputedStyle(document.body);
	expect(style.getPropertyValue("background")).toBe(
		"url(\"//example.com/image.png\")"
	);
	const bodyRule = Array.from(document.styleSheets)
		.flatMap(sheet => Array.from(sheet.cssRules))
		.find(rule => rule.selectorText === "body");
	expect(bodyRule).toBeDefined();
	expect(bodyRule.style.getPropertyValue("background-image")).toBe(
		"url(\"http://example.com/image.png\")"
	);
	await new Promise(resolve => setTimeout(resolve, 200));
	const importedBodyRule = Array.from(document.styleSheets)
		.flatMap(sheet => Array.from(sheet.cssRules))
		.flatMap(rule => rule.styleSheet ? Array.from(rule.styleSheet.cssRules) : [])
		.find(rule => rule.selectorText === "body");
	expect(importedBodyRule).toBeDefined();
	expect(importedBodyRule.style.getPropertyValue("color")).toBe("green");
});
