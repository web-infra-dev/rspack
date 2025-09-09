var value = require("../file");
import("./async.js"); // make sure ensure chunk runtime added
it("should accept a dependencies multiple times", done => {
	expect(value).toBe(1);
});
