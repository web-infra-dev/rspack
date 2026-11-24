import { local } from "./barrel.js";

it("should compile a local export alongside an unused `import defer` reexport", () => {
	expect(local).toBe(42);
});
