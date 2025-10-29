import m from "./module";

it("should add and remove chunks", async () => {
	let chunk = await m();
	expect(chunk.value).toBe(1);
	await NEXT_HMR();
	chunk = await m();
	expect(chunk.value).toBe(2);
	await NEXT_HMR();
	chunk = await m();
	expect(chunk.value).toBe(3);
});


module.hot.accept("./module");