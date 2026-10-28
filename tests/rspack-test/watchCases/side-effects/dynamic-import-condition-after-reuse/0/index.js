import { load, live } from "./module";

it("should keep the dynamic import while its export is used", async () => {
	expect(live).toBe("live");
	expect((await load()).value).toBe("feature");
});
