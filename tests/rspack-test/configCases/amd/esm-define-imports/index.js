import defaultValue, { named } from "./lib";
import * as namespace from "./lib";

it("should track default imports inside an external define callback", () => {
	expect(
		define(function () {
			return defaultValue;
		})
	).toBe(42);
});

it("should track named imports inside an external define arrow callback", () => {
	expect(define(() => named)).toBe("named-value");
});

it("should walk all define arguments without treating them as AMD dependencies", () => {
	expect(
		define(named, ["external-dependency"], function () {
			return [defaultValue, namespace.named];
		})
	).toEqual([42, "named-value"]);
});
