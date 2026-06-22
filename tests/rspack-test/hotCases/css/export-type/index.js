import textStyle from "./text.css";

it("should handle HMR for exportType", async () => {
	expect(typeof textStyle).toBe("string");
	expect(textStyle).toContain("color: red");
	expect(textStyle).toContain("text-class");
	expect(textStyle).toContain("imported-class");

	const { default: hi } = await import("./hi.txt", { with: { type: "text" }});
	expect(hi).toBe("hi");

	const sheetStyle = await import("./stylesheet.css", { with: { type: "css" }});
	expect(sheetStyle.default).toBeInstanceOf(CSSStyleSheet);
	const rules = Array.from(sheetStyle.default.cssRules);
	const rule = rules.find(r => r.selectorText.includes("sheet-class"));
	expect(rule).toBeDefined();
	expect(rule.style.color).toBe("green");

	module.hot.accept(["./text.css", "./stylesheet.css", "./hi.txt"]);

	await NEXT_HMR();
});

module.hot.accept();
