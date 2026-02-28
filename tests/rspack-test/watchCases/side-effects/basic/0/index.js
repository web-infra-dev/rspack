import {value} from "./module";

it("should have correct export from re-exports", () => {
	expect(value).toBe("foo");
});
