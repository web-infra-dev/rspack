import "./style-root.style.css";

const STATS = __STATS__.children[__STATS_I__];

it("should fold every style-export module into a single concatenated module", () => {
	const concatModules = STATS.modules.filter((m) => m.modules);
	expect(concatModules.length).toBe(1);
	expect(concatModules[0].modules.length).toBeGreaterThanOrEqual(2);
});
