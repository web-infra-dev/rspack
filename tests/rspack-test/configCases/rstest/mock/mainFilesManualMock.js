import { value } from "./src/main-file";

rs.mock("./src/main-file");

it("should resolve directory manual mock with configured mainFiles", () => {
	expect(value).toBe("mocked_main_file_main");
});
