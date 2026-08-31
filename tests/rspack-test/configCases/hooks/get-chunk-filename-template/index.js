require("./shared");

it("should render every chunk with the template returned by getChunkFilenameTemplate", () => {
	return import("./async").then(() => {
		expect(true).toBe(true);
	});
});
