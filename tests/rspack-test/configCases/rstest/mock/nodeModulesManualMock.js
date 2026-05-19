import { value } from "mocked-package";

rs.mock("mocked-package");

it("should mock node_modules package via manualMockRoot", () => {
	expect(value).toBe("mocked_package_value");
});
