import { formatDtae as format } from "./dates";

it("should report nearby exports for a missing name", () => {
	expect(format).toBeUndefined();
});
