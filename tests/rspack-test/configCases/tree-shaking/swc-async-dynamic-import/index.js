import { loadDestructured, loadMember } from "./loaders";

it("should tree shake dynamic imports lowered by the builtin SWC loader", async () => {
	const destructured = await loadDestructured();
	expect(destructured.value).toBe(3);
	expect(destructured.usedExports).toEqual(["default", "usedExports"]);

	const member = await loadMember();
	expect(member.value).toBe(1);
	expect(member.usedExports).toEqual(["a", "usedExports"]);
});
