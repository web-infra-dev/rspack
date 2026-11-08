import sheet from "./sheet-only.sheet.css";

const STATS = __STATS__.children[__STATS_I__];

it("should concatenate a css-style-sheet-export css module", () => {
	expect(sheet).toBeInstanceOf(CSSStyleSheet);
	expect(sheet._cssText).toContain(".sheet-only");
	// @charset prepended at byte 0.
	expect(sheet._cssText.match(/@charset/g)).toEqual(["@charset"]);
	expect(sheet._cssText.startsWith('@charset "UTF-8";\n')).toBe(true);
});

it("should fold the css-style-sheet module into a single concatenated module", () => {
	const concatModules = STATS.modules.filter((m) => m.modules);
	expect(concatModules.length).toBe(1);
	// index-css-style-sheet.js + sheet-only = 2
	expect(concatModules[0].modules.length).toBeGreaterThanOrEqual(2);
});
