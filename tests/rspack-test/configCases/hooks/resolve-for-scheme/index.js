import { value } from "./fake.js!=!data:text/javascript;charset=utf-8,export const value = 42;";
import source from "probe:original?original#original";

it("should have correct value", () => {
	expect(value).toBe(42);
});

it("should match rules against the resolved scheme components", () => {
	expect(source).toMatch(/export default ['"]resolved['"]/);
});
