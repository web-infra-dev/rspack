it("should handle mixed async blocks correctly (import and require.ensure)", () => new Promise((resolve, reject) => {
	const done = err => (err ? reject(err) : resolve());
	let count = 0;
	function next() {
		count++;
		if (count === 2) done();
	}

	import(/* webpackChunkName: "shared-async-block" */ "./module-a").then(moduleA => {
		expect(moduleA.default).toBe("a");
		next();
	}).catch(done);

	require.ensure(["./module-b"], function(require) {
		try {
			const moduleB = require("./module-b");
			expect(moduleB).toBe("b");
			next();
		} catch (e) {
			done(e);
		}
	}, "shared-async-block");
}));

it("should load an import nested inside require.ensure", () => new Promise((resolve, reject) => {
	require.ensure([], () => {
		import("./module-a").then(moduleA => {
			expect(moduleA.default).toBe("a");
		}).then(resolve, reject);
	}, error => reject(error));
}));
