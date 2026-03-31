it("should load the shared chunk", async () => {
	const { default: value } = await import(
		/* webpackChunkName: "shared" */ "./shared"
	);

	expect(value).toBe("shared");
});

export const version = "step-0";
