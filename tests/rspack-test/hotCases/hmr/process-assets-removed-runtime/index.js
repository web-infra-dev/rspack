let active = require("./a");

it("should preserve a shared chunk until its final runtime is removed", async () => {
	expect(await active).toBe("shared");
	await NEXT_HMR();
	active = require("./a");
	expect(active).toBe("removed");
});

module.hot.accept("./a");
