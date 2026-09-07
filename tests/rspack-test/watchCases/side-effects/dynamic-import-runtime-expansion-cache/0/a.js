import * as values from "./shared";

it("should leave the dynamic import unused in runtime a", () => {
	expect(values.live).toBe("live");
});
