import { live } from "./module";
import { leaf } from "./leaf";

it("should omit the eager dynamic import when its export becomes unused again", () => {
	expect(live).toBe("live");
	expect(leaf).toBe("after");
});
