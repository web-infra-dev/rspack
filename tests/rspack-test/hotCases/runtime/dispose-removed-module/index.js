var m = require("./module");

it("should dispose a module which is removed from bundle", () => new Promise((resolve, reject) => {
	const done = err => (err ? reject(err) : resolve());
	var disposed = [];
	m.setHandler((id) => {
		disposed.push(id);
	});
	NEXT(require("@rspack/test-tools/helper/legacy/update")(done, true, () => {
		require("./module");
		NEXT(require("@rspack/test-tools/helper/legacy/update")(done, true, () => {
			var newModule = require("./module");
			expect(disposed).toEqual([newModule.default]);
			done();
		}));
	}));
}));

if (module.hot) {
	module.hot.accept("./module");
}
