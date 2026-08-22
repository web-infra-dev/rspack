import { answer } from "./nice";
import { esmAnswer } from "./esm";

it("should only opt CommonJS modules out of concatenation", () => {
	expect(answer).toBe(42);
	expect(esmAnswer).toBe(43);
	const concatModules = __STATS__.modules.filter((m) => m.modules);
	expect(concatModules.length).toBe(1);
	expect(concatModules[0].modules.length).toBe(2);
	const module = __STATS__.modules.find((m) => m.name === "./nice.js");
	expect(module.optimizationBailout).toContainEqual(
		expect.stringContaining("not an ECMAScript module")
	);
});
