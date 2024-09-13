import {value} from "./module";

it("should have correct export from re-exports", function () {
	expect(value).toBe("foo");
});
