import { live } from "./module";

it("should remove the dynamic import after its export becomes unused", () => {
	expect(live).toBe("live");
});
