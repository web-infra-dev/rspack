import css from "./style.css";

it("should preserve distinct render conditions for repeated imports", () => {
	const normalized = css.replace(/\s+/g, " ").replace(/\s*\{/g, " {");
	for (const [media, supports, layer] of [
		["screen", "display: grid", "screen-layer"],
		["print", "display: grid", "screen-layer"],
		["screen", "display: flex", "screen-layer"],
		["screen", "display: grid", "other-layer"]
	]) {
		expect(normalized).toContain(`@media ${media} { @supports (${supports}) { @layer ${layer} { .shared { color: red; } } } }`);
	}
	expect(css.match(/\.shared/g)).toHaveLength(4);
});
