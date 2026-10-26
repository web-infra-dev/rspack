import { value, helper, unusedHook } from "./local";
console.log(value);
it("runs a newly reachable hook and updates its constant", () => {
	expect(value).toBe(Number(WATCH_STEP) === 2 ? "edited" : "final");
	expect(helper()).toBe("helper");
	expect(unusedHook()).toBe(`hook:${value}`);
});
