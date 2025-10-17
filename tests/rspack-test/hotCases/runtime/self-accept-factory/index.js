it("should able to accept for another module", async () => {
	const p = new Promise(resolve => require("./a")(resolve))
	await NEXT_HMR();
	await p;
});

module.hot.accept("./a");
