import value from "./uncached-loader!./value";
import stable from "./stable";

it("should rebuild an uncacheable module and reuse the other modules", () => {
	expect(value).toBe(COMPILER_INDEX + 1);
	expect(stable).toBe("stable");
});
