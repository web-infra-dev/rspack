import { usePathname } from "./navigation";

it("should not fold undefined == null to false", () => {
	expect(usePathname()).toBe("/");
});
