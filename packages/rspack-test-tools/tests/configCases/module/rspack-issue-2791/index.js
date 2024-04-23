import v1 from "./answer";
import v2 from "./answer?another-query";
import v3 from "./no-query-answer?raw";

it("should match resourceQuery and return string", () => {
	// query of target
	expect(v1).toBe("export const answer = 42;\n");
	// should use the query of target in alias
	expect(v2).toBe("export const answer = 42;\n");
	// the the query in request
	expect(v3).toBe("export const answer = 42;\n");
});
