import { dirname } from "./child/child";

it("dirname mock", function () {
	expect(__dirname).toBe("");
	expect(dirname).toBe("child");
});
