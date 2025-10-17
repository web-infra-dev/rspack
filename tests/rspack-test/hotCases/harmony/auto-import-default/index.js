import value from "./file";

it("should auto-import an ES6 imported default value from non-ESM module on accept", async () => {
	expect(value).toBe(1);
	await NEXT_HMR();
	expect(value).toBe(2);
	outside();
});

function outside() {
	expect(value).toBe(2);
}

module.hot.accept("./file");
