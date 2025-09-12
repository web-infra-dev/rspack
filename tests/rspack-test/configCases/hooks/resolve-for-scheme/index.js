import { value } from "./fake.js!=!data:text/javascript;charset=utf-8,export const value = 42;";

it("should have correct value", () => {
	expect(value).toBe(42);
});
