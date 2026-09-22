it("should run an entry that lists a stylesheet next to its script", () => {
	expect(Object.keys(__webpack_modules__)).toEqual(["./index.js"]);
	const computedStyle = getComputedStyle(document.body);
	expect(computedStyle.getPropertyValue("background")).toBe("red");
});
