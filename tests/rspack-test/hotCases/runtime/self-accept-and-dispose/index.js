it("should accept itself and pass data", async () => {
	const p = new Promise(resolve => require("./file")(resolve))
	await NEXT_HMR();
	await p;
});

module.hot.accept("./file");