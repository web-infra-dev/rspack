it("should allow mangling for cjs property reexports", () => {
	expect(require("./module-exports-property").aaa).toBe("aaa");
	expect(require("./module-exports-property").aaaCanMangle).toBe(true);
	expect(require("./module-exports-property").aaaReexportedCanMangle).toBe(true);
	expect(require("./module-exports-property").usedExports).toEqual(["aaa","aaaCanMangle","usedExports"]);
});

it("should not allow mangling for whole module.exports reexports", () => {
	expect(require("./module-exports-all").aaa).toBe("aaa");
	expect(require("./module-exports-all").aaaCanMangle).toBe(false);
	expect(require("./module-exports-all").aaaReexportedCanMangle).toBe(false);
	expect(require("./module-exports-all").usedExports).toEqual(["aaa","aaaCanMangle","aaaReexportedCanMangle", "usedExports"]);
});
