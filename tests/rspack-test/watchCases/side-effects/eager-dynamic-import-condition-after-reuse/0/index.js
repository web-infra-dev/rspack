import { load, live } from "./module";

it("should keep the eager import while its export is used", async () => {
	expect(live).toBe("live");
	expect((await load()).value).toBe("EAGER_CONDITIONAL_FEATURE_MARKER");
});
