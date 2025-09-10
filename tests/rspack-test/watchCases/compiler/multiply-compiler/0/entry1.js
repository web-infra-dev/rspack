import module from "./file";

it("should watch for multiply compiler (entry1)", function () {
	expect(module).toBe(WATCH_STEP);
});
