import api, * as middle from "./middle";
import * as source from "./source";
import cycleDefault from "./cycle-middle";
import * as cycleSource from "./cycle-source";

it("should export the imported namespace object as default", () => {
	expect(api).toBe(source);
	expect(middle.default).toBe(source);
	expect(api.default).toBe("default");
	expect(api.named).toBe("named");
});

it("should follow imported-specifier reexport behavior in cycles", () => {
	expect(cycleDefault).toBe(cycleSource);
	expect(cycleSource.observed).toBeUndefined();
});
