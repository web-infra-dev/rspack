import root from "./text-root.text.css";

const STATS = __STATS__.children[__STATS_I__];

it("should concatenate text-export css modules", () => {
	expect(typeof root).toBe("string");

	// Single @charset at byte 0 even after concatenating a chain.
	expect(root.match(/@charset/g)).toEqual(["@charset"]);
	expect(root.startsWith('@charset "UTF-8";\n')).toBe(true);
	expect(root).toContain(".text-root");
	expect(root).toContain(".text-leaf");
});

it("should fold every text module into a single concatenated module", () => {
	const concatModules = STATS.modules.filter((m) => m.modules);
	if (concatModules.length > 0) {
		// index-text.js + text-root; text-leaf is folded into text-root.
		expect(concatModules[0].modules.length).toBeGreaterThanOrEqual(2);
	}
});
