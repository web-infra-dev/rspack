import shared from "./shared";

const direct = shared.base;

it("should keep a single __webpack_require__ export on runtimeChunk false entry chunks", async () => {
	const mod = await import("./dynamic");

	expect(mod.value).toBe(42);
	expect(direct).toBe(1);
});

export { direct };
