import "./inject";

import foo from "foo";
const bar = require("bar");
const react = require("react");

it("should work with array type of externals", function () {
	expect(foo).toBe("foo");
	expect(bar).toBe("bar");
});
