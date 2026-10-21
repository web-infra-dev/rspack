import value from "./module";

it("should return the promise from an ESM accept callback", async () => {
	expect(value).toBe(1);
	let completed = false;
	module.hot.accept("./module", () => new Promise(resolve => {
		setTimeout(() => {
			completed = true;
			resolve();
		}, 0);
	}));
	await NEXT_HMR();
	expect(value).toBe(2);
	expect(completed).toBe(true);
});
