import { textA, textB, sheetA, sheetB, linkAClass, linkBClass, styleA, styleB } from "./lib.js";

it("should handle HMR for all exportTypes with concatenation", async () => {
	expect(typeof textA).toBe("string");
	expect(textA).toContain("color: red");
	expect(textA).toContain("font-size: 12px");
	expect(typeof textB).toBe("string");
	expect(textB).toContain("color: blue");

	expect(sheetA).toBeInstanceOf(CSSStyleSheet);
	expect(sheetA._cssText).toContain("color: green");
	expect(sheetA._cssText).toContain("font-weight: bold");
	expect(sheetB).toBeInstanceOf(CSSStyleSheet);
	expect(sheetB._cssText).toContain("color: purple");

	expect(typeof styleA).toBe("string");
	expect(typeof styleB).toBe("string");
	const allStyles = () =>
		Array.from(window.document.getElementsByTagName("style")).map(s => s.textContent);
	expect(allStyles().some(c => c.includes("color: red"))).toBe(true);
	expect(allStyles().some(c => c.includes("font-size: 12px"))).toBe(true);
	expect(allStyles().some(c => c.includes("color: blue"))).toBe(true);

	expect(typeof linkAClass).toBe("string");
	expect(linkAClass.length).toBeGreaterThan(0);
	expect(typeof linkBClass).toBe("string");
	expect(linkBClass.length).toBeGreaterThan(0);
	const allLinks = () => [...window.document.getElementsByTagName("link")];
	const allLinkCss = () => allLinks().map(l => getLinkSheet(l)).join("\n");
	expect(allLinkCss()).toContain("text-align: center");
	expect(allLinkCss()).toContain("letter-spacing: 1px");
	expect(allLinkCss()).toContain("text-align: right");

	await NEXT_HMR();

	expect(textA).toContain("font-size: 14px");
	expect(textB).toContain("color: cyan");
	expect(sheetA._cssText).toContain("font-weight: normal");
	expect(sheetB._cssText).toContain("color: violet");
	expect(typeof styleA).toBe("string");
	expect(typeof styleB).toBe("string");
	expect(allStyles().some(c => c.includes("font-size: 12px"))).toBe(false);
	expect(allStyles().some(c => c.includes("color: blue"))).toBe(true);
	expect(allLinkCss()).toContain("letter-spacing: 2px");
	expect(allLinkCss()).toContain("text-align: left");
});

module.hot.accept(["./lib.js"]);
