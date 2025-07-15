import d2 from "#internal";
import d1 from "./pkg.mjs";

it("imports field to resolve to the same", () => {
	expect(d1).toBe(d2);
});
