import { value } from "package-with-manual-mock";

rs.mock("package-with-manual-mock");

it("should resolve a dependency mock from the project root", () => {
	expect(value).toBe("root_mocked_package_entry");
});
