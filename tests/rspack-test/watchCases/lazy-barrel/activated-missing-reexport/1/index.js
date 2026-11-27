import { a, b } from "./pkg/named";

it("should report the re-export once a later importer requests it", () => {
	expect(a).toBe("named");
	expect(b).toBe(undefined);
});
