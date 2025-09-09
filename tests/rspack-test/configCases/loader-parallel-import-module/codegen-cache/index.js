import v1 from "./app-proxy";
import v2 from "./app";

it("should compile", () => {
	expect(v1).toBe(42);
	expect(v2).toBe(42);
});
