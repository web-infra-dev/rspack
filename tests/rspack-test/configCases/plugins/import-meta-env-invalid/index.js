it("should replace invalid import.meta.env values with an empty object", () => {
	expect(import.meta.env).toEqual({});
});
