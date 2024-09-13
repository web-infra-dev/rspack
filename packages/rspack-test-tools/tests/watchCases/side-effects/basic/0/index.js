import {value} from "./module";

it("should have correct export from re-exports", function () {
	console.log(value)
	expect(value).toBe("foo");
});
