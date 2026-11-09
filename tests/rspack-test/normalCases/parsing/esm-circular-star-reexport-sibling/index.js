import * as barrel from "./barrel";

it("should resolve cyclic star exports through the sibling that provides the binding", () => {
	expect(barrel).toEqual(
		nsObj({
			direct: 1,
			indirect: 2,
			local: 3,
			renamed: 3,
			namespace: nsObj({ direct: 1, indirect: 2 })
		})
	);
});
