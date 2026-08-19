import "../import-with-default";
import placeholder from "./placeholder.json";

it("should not replace placeholder-like strings in source content", () => {
	expect(placeholder).toEqual({
		value: "__rspack_generated_top_level_symbol_0__"
	});
});
