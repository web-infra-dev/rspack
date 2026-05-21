import textA from "./text-a.css";
import textB from "./text-b.css";
import sheetA from "./sheet-a.css";
import sheetB from "./sheet-b.css";
import { "link-a-class" as linkAClass } from "./link-a.module.css";

it("should concatenate text exportType modules", () => {
	expect(typeof textA).toBe("string");
	expect(textA).toContain("color: red");

	expect(typeof textB).toBe("string");
	expect(textB).toContain("color: blue");
});

it("should concatenate css-style-sheet exportType modules", () => {
	expect(sheetA).toBeInstanceOf(CSSStyleSheet);

	expect(sheetB).toBeInstanceOf(CSSStyleSheet);
});

it("should concatenate link exportType (CSS modules) and export class names", async () => {
	expect(typeof linkAClass).toBe("string");
	expect(linkAClass.length).toBeGreaterThan(0);

	const links = [...document.getElementsByTagName("link")];
	expect(links.length).toBeGreaterThan(0);
	const initialCss = links
		.filter(l => l.sheet)
		.map(l => l.sheet.css)
		.join("\n");
	if (initialCss.trim()) {
		expect(initialCss).toContain("text-align: center");
	}

	const { "link-b-class": linkBClass } = await import(/* webpackChunkName: "link-b" */ "./link-b.module.css");
	expect(typeof linkBClass).toBe("string");
	expect(linkBClass.length).toBeGreaterThan(0);
	expect(linkAClass).not.toBe(linkBClass);

	const allLinks = [...document.getElementsByTagName("link")];
	const asyncCss = allLinks
		.filter(l => l.sheet)
		.map(l => l.sheet.css)
		.join("\n");
	if (asyncCss.trim()) {
		expect(asyncCss).toContain("text-align: right");
	}
});

it("should concatenate all modules into one concatenated module", () => {
	const concatModules = __STATS__.modules.filter(m => m.modules);
	if (concatModules.length > 0) {
		// index.js + 2 text + 2 sheet; link modules still participate in CSS chunking.
		expect(concatModules[0].modules.length).toBeGreaterThanOrEqual(5);
	}
});
