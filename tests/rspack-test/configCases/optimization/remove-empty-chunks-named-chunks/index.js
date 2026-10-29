import a from "./a";

it("should load a named chunk that was removed as empty", async () => {
	const lazy = await import(/* webpackChunkName: "empty" */ "./a");
	expect(lazy.default).toBe(a);
});
