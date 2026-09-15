it("should handle import.meta.rspackRsc as an unknown import.meta property without RSC plugins", () => {
	expect(import.meta.rspackRsc).toBe(undefined);
	expect(typeof import.meta.rspackRsc).toBe("undefined");
});
