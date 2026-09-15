import { value } from "package-with-manual-mock";

rs.mock("package-with-manual-mock");

it("should fall back to the npm package entry adjacent mock", () => {
	expect(value).toBe("package_mock");
});
