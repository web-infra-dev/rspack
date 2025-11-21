var m = require("./module");

it("should dispose a module which is removed from bundle", async () => {
	var disposed = [];
	m.setHandler((id) => {
		disposed.push(id);
	});
	await NEXT_HMR();
	require("./module");
	await NEXT_HMR();
	var newModule = require("./module");
	expect(disposed).toEqual([newModule.default]);
});

module.hot.accept("./module");
