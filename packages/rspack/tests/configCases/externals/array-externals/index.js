import "./inject";

import foo from "foo";
import raz from "raz";
const myos = require("myos");
const bar = require("bar");
const baz = require("baz");

it("should work with array type of externals", function () {
	expect(foo).toBe("foo");
	expect(bar).toBe("bar");
	expect(baz).toBe("baz");
	expect(raz).toBe("raz");
	expect(typeof myos.constants.errno.EBUSY).toBe("number");
});
