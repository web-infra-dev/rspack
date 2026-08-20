import { factoryValue } from "./barrel";

const findModule = name => __STATS__.modules.find(m => m.name.endsWith(`/${name}`));

it("should keep exports unknown when module factories are exposed", () => {
	expect(factoryValue).toBe("factory");
	expect(findModule("empty.js").providedExports).toBe(null);
});
