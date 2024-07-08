import "./foo";

import("./async").then(() => {
	it("ok", done => {
		done();
	});
});
