it("should keep the wasm file name when its module moves into a new shared chunk", async () => {
	const [{ default: a }, { default: b }] = await Promise.all([import("./a"), import("./b")]);
	expect(a).toBe(3);
	expect(b).toBe(0);
	const stats = await NEXT_HMR();
	const completeUpdates = stats.assets.map(asset => asset.name).filter(name => name.includes(".full."));
	expect(completeUpdates).toHaveLength(1);
	expect(completeUpdates[0]).toMatch(/^shared\..+\.hot-update\.js\.full\.js$/);
	const [{ default: nextA }, { default: nextB }] = await Promise.all([import("./a"), import("./b")]);
	expect(nextA).toBe(3);
	expect(nextB).toBe(3);
});
