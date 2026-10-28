import { value as linked } from "linked-manual-mock";
import { value as aliased } from "@/package";
import { value as subpath } from "package-with-manual-mock/feature";
import { value as fallback } from "package-with-manual-mock/unmocked";

rs.mock("linked-manual-mock");
rs.mock("@/package");
rs.mock("package-with-manual-mock/feature");
rs.mock("package-with-manual-mock/unmocked");

it("should find adjacent mocks for linked packages, aliases and package subpaths", () => {
	expect(linked).toBe("mocked_package_entry");
	expect(aliased).toBe("mocked_package_entry");
	expect(subpath).toBe("mocked_feature");
});

it("should retain automock fallback when neither manual mock exists", () => {
	expect(fallback).toBe("automock_fallback");
});
