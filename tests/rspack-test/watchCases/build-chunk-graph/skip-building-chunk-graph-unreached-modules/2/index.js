import { value, helper, unusedHook } from "./local";

it("runs a newly reachable hook", () => {
	expect(value).toBe("edited");
	expect(helper()).toBe("helper");
	expect(unusedHook()).toBe("hook:edited");
});
