import "./bridge";

it("should recover pure modules after cache rebuild", async () => {
	if (COMPILER_INDEX === 0) await NEXT_START();
});
