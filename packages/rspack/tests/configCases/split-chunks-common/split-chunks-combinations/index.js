import path from "path";

import(/* webpackChunkName: "async-a" */ "./a");
import(/* webpackChunkName: "async-b" */ "./b");
import(/* webpackChunkName: "async-c" */ "./c");
import(/* webpackChunkName: "async-d" */ "./d");
import(/* webpackChunkName: "async-e" */ "./e");
import(/* webpackChunkName: "async-f" */ "./f");
import(/* webpackChunkName: "async-g" */ "./g");

it("worked", done => {
	const shared = require("./~x_js~y_js.js");
	expect(shared.modules["./x.js"]).toBeDefined();
	expect(shared.modules["./y.js"]).toBeDefined();

	const asyncA = require("./async-a.js");
	expect(asyncA.modules["./x.js"]).toBeUndefined();
	expect(asyncA.modules["./y.js"]).toBeUndefined();

	const asyncC = require("./async-c.js");
	expect(asyncC.modules["./x.js"]).toBeDefined();
	expect(asyncC.modules["./y.js"]).toBeUndefined();

	done();
});
