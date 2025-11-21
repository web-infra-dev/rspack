import { value } from "./shared";
it("should work with chunkLoading=require", function () {
	expect(value).toBe("123");
});
