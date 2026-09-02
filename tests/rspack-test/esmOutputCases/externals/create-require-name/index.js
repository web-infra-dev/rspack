import { createRequire } from "node:module";

export const createRequireName = createRequire.name;

it("should preserve a collision-free external import name", () => {
	expect(createRequireName).toBe("createRequire");
});
