import dep from "dep";
import depWithSpace from "dep-with-space";
import depWithoutAuthority from "dep-without-authority";

it("should locate packages with absolute file URLs", () => {
	expect(dep).toBe("store/dep");
});

it("should decode the file URL of a package", () => {
	expect(depWithSpace).toBe("store/dep with space");
});

it("should locate packages with a file URL that has no authority", () => {
	expect(depWithoutAuthority).toBe("store/dep-without-authority");
});
