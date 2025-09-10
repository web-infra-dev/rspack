import path from "path";

import(/* webpackChunkName: "async-a" */ "./a");
import(/* webpackChunkName: "async-b" */ "./b");
import(/* webpackChunkName: "async-c" */ "./c");
import(/* webpackChunkName: "async-d" */ "./d");
import(/* webpackChunkName: "async-e" */ "./e");
import(/* webpackChunkName: "async-f" */ "./f");
import(/* webpackChunkName: "async-g" */ "./g");

function existForModule(chunk, module) {
	const { modules } = chunk;
	return module in modules;
}

it("worked", done => {
	expect(existForModule(require("./x_js-y_js.js"), "./x.js")).toBe(true);
	expect(existForModule(require("./x_js-y_js.js"), "./y.js")).toBe(true);

	expect(existForModule(require("./async-a.js"), "./x.js")).toBe(false);
	expect(existForModule(require("./async-a.js"), "./y.js")).toBe(false);

	expect(existForModule(require("./async-b.js"), "./x.js")).toBe(false);
	expect(existForModule(require("./async-b.js"), "./y.js")).toBe(false);

	expect(existForModule(require("./async-c.js"), "./x.js")).toBe(true);
	expect(existForModule(require("./async-c.js"), "./y.js")).toBe(false);

	expect(existForModule(require("./async-d.js"), "./x.js")).toBe(true);
	expect(existForModule(require("./async-d.js"), "./y.js")).toBe(false);

	expect(existForModule(require("./async-e.js"), "./x.js")).toBe(true);
	expect(existForModule(require("./async-e.js"), "./y.js")).toBe(false);

	expect(existForModule(require("./async-f.js"), "./x.js")).toBe(true);
	expect(existForModule(require("./async-f.js"), "./y.js")).toBe(false);

	expect(existForModule(require("./async-g.js"), "./x.js")).toBe(true);
	expect(existForModule(require("./async-g.js"), "./y.js")).toBe(false);

	done();
});
