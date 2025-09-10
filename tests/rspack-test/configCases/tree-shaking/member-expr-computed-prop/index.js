import { answer, answerUsed } from "./lib";
it("should only import assets that included in chunks", () => {
	const res = {};
	res[answer] = 1;
	expect(answerUsed).toBe(true);
});
