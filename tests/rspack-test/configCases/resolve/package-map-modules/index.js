import local from "local";
import localFile from "local/file";
import dep from "dep";

it("should keep resolving from the other module directories", () => {
	expect(local).toBe("modules/local");
	expect(localFile).toBe("modules/local/file");
});

it("should use the package map in place of node_modules", () => {
	expect(dep).toBe("store/dep");
});
