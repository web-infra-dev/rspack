import binary from "data:application/octet-stream;base64,AP9h";
import emptyBinary from "data:application/octet-stream;base64,";

it("preserves Unicode text and binary sources", () => {
	expect("你好 🌍").toHaveLength(5);
	expect(binary).toBe("data:application/octet-stream;base64,AP9h");
	expect(emptyBinary).toBe("data:application/octet-stream;base64,");
});
