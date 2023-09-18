import fakeValue from "fake";
import fileValue from "@/file";
import baseUrlFileValue from "src/file";
import fileValueWithQuery1 from "@/file?a=b";
import fileValueWithQuery2 from "@/file?a=c";

it("should require real files by alias tsconfig paths", () => {
	expect(fakeValue).toEqual("real");
	expect(fileValue).toStrictEqual({});
});

it("absolute path relative base url should works", () => {
	expect(baseUrlFileValue).toStrictEqual({});
	expect(baseUrlFileValue === fileValue).toBe(true);
	expect(fileValueWithQuery1).toStrictEqual({});
	expect(fileValueWithQuery2).toStrictEqual({});
	expect(fileValueWithQuery1 !== fileValueWithQuery2).toBe(true);
	expect(fileValueWithQuery1 !== baseUrlFileValue).toBe(true);
});
