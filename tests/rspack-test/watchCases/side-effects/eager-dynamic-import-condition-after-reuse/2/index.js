import { live } from "./module";

it("should remove the eager import after its export becomes unused", () => {
	expect(live).toBe("live");
});
