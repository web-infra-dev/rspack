import binary from "data:application/octet-stream;base64,AP9h";

it("preserves Unicode text and binary sources", () => {
	expect("你好 🌍").toHaveLength(5);
	expect(binary).toBe("data:application/octet-stream;base64,AP9h");
});
