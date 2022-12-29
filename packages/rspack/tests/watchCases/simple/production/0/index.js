import module from "./changing-module";

it("should watch for changes", function () {
	expect(module).toBe(WATCH_STEP);
});
