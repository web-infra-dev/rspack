import { url } from "@generated/client";

it("should resolve import.meta.url through tsconfig paths with an absolute or relative config file", () => {
	expect(url).toBe(EXPECTED_URL);
});
