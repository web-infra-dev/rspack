import lib3 from "./lib";
import lib1 from "./lib?source";
import lib2 from "./lib#source";

it("should have empty resourceQuery and resourceFragment when resource without ends with ?xx or #xx", () => {
	expect(lib1).toBe("afragmentloader");
	expect(lib2).toBe("aqueryloader");
	expect(lib3).toBe("afragmentloaderqueryloader");
});
