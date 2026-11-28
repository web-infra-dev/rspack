it("should keep the asset file name when its module moves into a new shared chunk", async () => {
	const [{ default: a }, { default: b }] = await Promise.all([import("./a"), import("./b")]);
	expect(a).toMatch(/logo\.[0-9a-f]{8}\.svg$/);
	expect(b).toBe("b");
	const stats = await NEXT_HMR();
	const completeUpdates = stats.assets.map(asset => asset.name).filter(name => name.includes(".full."));
	expect(completeUpdates).toHaveLength(1);
	expect(completeUpdates[0]).toMatch(/^shared\..+\.hot-update\.js\.full\.js$/);
	const [{ default: nextA }, { default: nextB }] = await Promise.all([import("./a"), import("./b")]);
	expect(nextA).toBe(a);
	expect(nextB).toBe(a);
});
