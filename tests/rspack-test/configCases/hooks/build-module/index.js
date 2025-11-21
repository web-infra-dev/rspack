import { a } from "./a.js";

it("should compile successfully with build-module", () => {
	expect(a).toBe(3);
});
