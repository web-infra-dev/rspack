import * as a from "./reexport";

it("should not suspend when assign depth for include dependencies", async () => {
	expect(a.a).toBe(NaN);
});
