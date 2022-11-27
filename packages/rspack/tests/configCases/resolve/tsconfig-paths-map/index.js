import fakeValue from "fake";
import fileValue from "@/file";

it("should require real files by alias tsconfig paths", () => {
	expect(fakeValue).toEqual("real");
	expect(fileValue).toEqual("src/file");
});
