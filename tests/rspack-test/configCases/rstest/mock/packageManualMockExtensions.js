import { value as shared } from "shared-format-mock";
import { value as exact } from "exact-format-mock";

rs.mock("shared-format-mock");
rs.mock("exact-format-mock");

it("should resolve a shared mock for an mjs entry", () => {
	expect(shared).toBe("shared_mock");
});

it("should prefer the mjs mock over extension fallback", () => {
	expect(exact).toBe("exact_import_mock");
});
