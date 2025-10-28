

it("should dispose a chunk which is removed from bundle", async () => {
	var m1 = require("./module");
	var x1 = await m1.default;
	expect(x1.default).toEqual("version a1");
	await NEXT_HMR();
	var m2 = require("./module");
	var x2 = await m2.default;
	expect(x2.default).toEqual("version b1");
	await NEXT_HMR();
	var m3 = require("./module");
	var x3 = await m3.default;
	expect(x3.default).toEqual("version b2");
	await NEXT_HMR();
	var m4 = require("./module");
	var x4 = await m4.default;
	expect(x4.default).toEqual("version a2");
	expect(x4).not.toEqual(x1);
});

module.hot.accept("./module");
