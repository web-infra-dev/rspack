import { readFileSync } from "node:fs";

rs.mock("node:fs");

it("should mock node builtins via manualMockRoot", () => {
	expect(readFileSync()).toBe("mocked_fs_read_file_sync");
});
