import module from "./file";

it("should watch for multiply compiler (entry2)", function () {
	expect(module).toBe(WATCH_STEP);
});
