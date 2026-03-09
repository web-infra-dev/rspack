const TOP_OF_FILE = 1;

it("should doMock A->B->C re-export chain", async () => {
	rs.doMock("reexport-top", () => {
		return {
			useParams: () => ({ id: "triple-do-mocked-id" })
		};
	});

	const { useParams } = await import("reexport-top");
	expect(useParams()).toEqual({ id: "triple-do-mocked-id" });
});
