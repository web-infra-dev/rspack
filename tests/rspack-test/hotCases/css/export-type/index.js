import textStyle from "./text.css";

it("should handle HMR for exportType", async () => {
	expect(typeof textStyle).toBe("string");
	expect(textStyle).toContain("color: red");
	expect(textStyle).toContain("text-class");

	const { default: hi } = await import("./hi.txt", { with: { type: "text" } });
	expect(hi).toBe("hi");

	const sheetStyle = await import("./stylesheet.css", { with: { type: "css" } });
	expect(sheetStyle.default).toBeInstanceOf(CSSStyleSheet);
	let rules = Array.from(sheetStyle.default.cssRules);
	let rule = rules.find(r => r.selectorText.includes("sheet-class"));
	expect(rule).toBeDefined();
	expect(rule.style.color).toBe("green");

	await NEXT_HMR();

	expect(typeof textStyle).toBe("string");
	expect(textStyle).toContain("text-class");

	const updatedSheetStyle = await import("./stylesheet.css", { with: { type: "css" } });
	expect(updatedSheetStyle.default).toBeInstanceOf(CSSStyleSheet);
	rules = Array.from(updatedSheetStyle.default.cssRules);
	rule = rules.find(r => r.selectorText.includes("sheet-class"));
	expect(rule).toBeDefined();
	expect(rule.style.color).toBe("green");
});

module.hot.accept(["./text.css", "./stylesheet.css", "./hi.txt"]);
