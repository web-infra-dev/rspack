import a from "./a"
import value from "./file"

const req = require.context("./foo", false, /^.*\.js$/);
it("should ignore change to file and directory", function() {
	expect(a).toBe(+WATCH_STEP);
	expect(req.keys()).toEqual(["./0.js"])
	expect(value).toBe(1);
});
