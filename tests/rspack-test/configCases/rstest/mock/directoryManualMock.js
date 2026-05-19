import { value } from "./src/common";

rs.mock("./src/common");

it("should resolve directory manual mock from nested __mocks__/index", () => {
	expect(value).toBe("mocked_common_index");
});
