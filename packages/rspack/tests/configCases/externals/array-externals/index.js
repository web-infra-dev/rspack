import "./inject";

import foo from "foo";
import raz from "raz";
const myos = require("myos");
const bar = require("bar");
const baz = require("baz");
const fn = require("fn");
const asyncFn = require("asyncFn");
const external = require("external");
const external2 = require("external2");
const external3 = require("external3");
const external4 = require("external4");
const external5 = require("external5");

it("should work with array type of externals", function () {
	expect(foo).toBe("foo");
	expect(bar).toBe("bar");
	expect(baz).toBe("baz");
	expect(raz).toBe("raz");
	expect(fn).toBe("fn");
	expect(asyncFn).toBe("asyncFn");
	expect(typeof myos.constants.errno.EBUSY).toBe("number");

	expect(external).toBe(Array.isArray);
	expect(external2).toBe(process.version);
	expect(external3).toBe(globalThis);
	expect(external4).toBe(global.process.version);
	expect(external5).toBe("yj");
});
