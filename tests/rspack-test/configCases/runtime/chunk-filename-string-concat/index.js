it("should preserve string concatenation for numeric chunk ids", async () => {
	const id = 12;
	const filename = __webpack_get_script_filename__(id);
	expect(typeof filename).toBe("string");
	expect(filename).toBe(
		CHUNK_FILENAME.replace(/\[id\]/g, String(id)).replace(
			"[fullhash]",
			__webpack_hash__
		)
	);
	expect((await import("./lazy")).value).toBe(42);
});
