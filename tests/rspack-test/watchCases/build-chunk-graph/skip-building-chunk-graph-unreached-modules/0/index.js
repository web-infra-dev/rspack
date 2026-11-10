import { value, helper } from "./local";

it("uses the updated constant through a barrel", () => {
	expect(value).toBe(Number(WATCH_STEP) === 0 ? "cold" : "edited");
	expect(helper()).toBe("helper");
});
