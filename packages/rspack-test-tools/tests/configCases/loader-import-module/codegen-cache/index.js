import v2 from "./app";
import v1 from "./app-proxy";

it("should compile", () => {
	expect(v1).toBe(42);
	expect(v2).toBe(42);
});
