import { P, Q } from "./module";

it("should preserve top-level bindings used as loop assignment targets", () => {
	expect(P).toBe("expected");
	expect(Q).toBe("expected");

	const source = require("fs").readFileSync(__filename, "utf-8");
	expect(source).toContain('var P = "initial"');
	expect(source).toContain('var Q = "initial"');
});
