import { value } from "./src/file-precedence/foo";

rs.mock("./src/file-precedence/foo");

it("should prefer file mocks over same-basename directories", () => {
	expect(value).toBe("mocked_file_precedence_file");
});
