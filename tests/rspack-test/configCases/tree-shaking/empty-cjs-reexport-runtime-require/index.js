import { runtimeValue } from "./barrel";

const findModule = name => __STATS__.modules.find(m => m.name.endsWith(`/${name}`));

it("should keep exports unknown when the runtime require can access them", () => {
	expect(runtimeValue).toBe("runtime");
	expect(findModule("empty.js").providedExports).toBe(null);
});
