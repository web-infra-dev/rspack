import shared from "./shared";

const direct = shared.base;

it("should load wrapped CommonJS with direct initializers on runtimeChunk false entry chunks", async () => {
	const mod = await import("./dynamic");

	expect(mod.value).toBe(42);
	expect(direct).toBe(1);
});

export { direct };
