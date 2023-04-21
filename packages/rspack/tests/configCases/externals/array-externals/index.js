import "./inject";

import foo from "foo";
import raz from "raz";
const myos = require("myos");
const bar = require("bar");
const baz = require("baz");
const fn = require("fn");
const asyncFn = require("asyncFn");

it("should work with array type of externals", function () {
	expect(foo).toBe("foo");
	expect(bar).toBe("bar");
	expect(baz).toBe("baz");
	expect(raz).toBe("raz");
	expect(fn).toBe("fn");
	expect(asyncFn).toBe("asyncFn");
	expect(typeof myos.constants.errno.EBUSY).toBe("number");
});
