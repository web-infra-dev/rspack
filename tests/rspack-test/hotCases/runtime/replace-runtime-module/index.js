import m from "./module";

it("should dispose a chunk which is removed from bundle", async () => {
	let a = await m;
	expect(a.default).toEqual("a");
	await NEXT_HMR();
	let b = await m;
	expect(b.default).toEqual("b");
});

module.hot.accept("./module");
