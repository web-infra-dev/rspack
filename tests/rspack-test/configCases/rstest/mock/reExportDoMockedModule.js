it("should doMock a re-exported module", async () => {
	rs.doMock("reexport-intermediate", () => {
		return {
			useParams: () => ({ id: "do-mocked-id" })
		};
	});

	const { useParams } = await import("reexport-intermediate");
	expect(useParams()).toEqual({ id: "do-mocked-id" });
});
