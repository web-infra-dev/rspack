import { read } from "./state";
import { sum, updateConst, writeConst, writeLet } from "./writer";
import { bump, bumpInExpression, bumpPostfix, bumpPrefix } from "./bump";
import { readCount, readValue } from "./counter";
import { updateMember, updateMemberPrefix, writeMember } from "./namespace";
import { readQuoted } from "./quoted";
import {
	updateThroughNamespace,
	writeAliased,
	writeThroughNamespace
} from "./quoted-writer";
import {
	loopAwaitIdentifier,
	loopInIdentifier,
	loopOfIdentifier,
	loopOfNamespaceMember
} from "./loops";
import { andAssign, orAssign, updateRenamed, writeRenamed } from "./logical";
import {
	loopNamespace,
	namespaceObject,
	reassignNamespace,
	updateNamespace
} from "./namespace-binding";
import {
	loopArrayPattern,
	loopObjectPattern,
	readAfterPattern,
	writeArrayPattern,
	writeImportedObjectProperty,
	writeNamespaceMemberPattern,
	writeNestedPattern,
	writeObjectPattern,
	writeObjectPatternDefault,
	writeRenamedObjectPattern,
	writeRestPattern
} from "./destructuring";
import {
	asiAssign,
	asiAssignReexportedDefault,
	asiKeepTruthyReexportedDefault,
	asiUpdate
} from "./asi";
import { writeCjsExport } from "./cjs-namespace";

it("should reject a write to an imported binding", () => {
	expect(writeLet).toThrow(TypeError);
	expect(writeConst).toThrow(TypeError);
	expect(read()).toBe(1);
	expect(sum()).toBe(103);
});

it("should reject a compound write and one in expression position", () => {
	expect(bump).toThrow(TypeError);
	expect(bumpInExpression).toThrow(TypeError);
	expect(readCount()).toBe(0);
});

it("should reject a logical assignment", () => {
	expect(orAssign).toThrow(TypeError);
	expect(andAssign).toThrow(TypeError);
	expect(readCount()).toBe(0);
	expect(readValue()).toBe(1);
});

it("should reject a write to a renamed import", () => {
	expect(writeRenamed).toThrow(TypeError);
	expect(updateRenamed).toThrow(TypeError);
	expect(readValue()).toBe(1);
});

it("should reject an update of an imported binding", () => {
	expect(bumpPostfix).toThrow(TypeError);
	expect(bumpPrefix).toThrow(TypeError);
	expect(updateConst).toThrow(TypeError);
	expect(readCount()).toBe(0);
	expect(sum()).toBe(103);
});

it("should reject a write through the namespace object", () => {
	expect(writeMember).toThrow(TypeError);
	expect(updateMember).toThrow(TypeError);
	expect(updateMemberPrefix).toThrow(TypeError);
	expect(readCount()).toBe(0);
});

it("should reject a write to an export no identifier can spell", () => {
	expect(writeThroughNamespace).toThrow(TypeError);
	expect(updateThroughNamespace).toThrow(TypeError);
	expect(writeAliased).toThrow(TypeError);
	expect(readQuoted()).toBe(1);
});

it("should reject a write to a loop's iteration binding", () => {
	expect(loopOfIdentifier).toThrow(TypeError);
	expect(loopInIdentifier).toThrow(TypeError);
	expect(loopOfNamespaceMember).toThrow(TypeError);
	expect(readCount()).toBe(0);
});

it("should reject it in an async iteration too", () =>
	expect(loopAwaitIdentifier()).rejects.toThrow(TypeError));

it("should reject a write to the namespace binding itself", () => {
	expect(reassignNamespace).toThrow(TypeError);
	expect(updateNamespace).toThrow(TypeError);
	expect(loopNamespace).toThrow(TypeError);
	expect(typeof namespaceObject()).toBe("object");
	expect(namespaceObject().count).toBe(0);
});

it("should reject a destructuring write", () => {
	expect(writeArrayPattern).toThrow(TypeError);
	expect(writeObjectPattern).toThrow(TypeError);
	expect(writeObjectPatternDefault).toThrow(TypeError);
	expect(writeRenamedObjectPattern).toThrow(TypeError);
	expect(writeNestedPattern).toThrow(TypeError);
	expect(writeRestPattern).toThrow(TypeError);
	expect(writeNamespaceMemberPattern).toThrow(TypeError);
	expect(loopArrayPattern).toThrow(TypeError);
	expect(loopObjectPattern).toThrow(TypeError);
	expect(readAfterPattern()).toBe(0);
	expect(readCount()).toBe(0);
});

it("should keep a destructuring write to an imported object's property", () => {
	expect(writeImportedObjectProperty()).toBe(8);
});

it("should reject a write in a statement that relies on ASI", () => {
	expect(asiAssign).toThrow(TypeError);
	expect(asiUpdate).toThrow(TypeError);
	expect(asiAssignReexportedDefault).toThrow(TypeError);
	expect(asiKeepTruthyReexportedDefault()).toBe(1);
	expect(readCount()).toBe(0);
});

it("should keep a write to a CommonJS module's export", () => {
	expect(writeCjsExport()).toBe(2);
});

it("should keep every module concatenated", () => {
	if (!CONCATENATED) return;
	const concatenated = __STATS__.children[__STATS_I__].modules.filter((m) => m.modules);
	expect(concatenated).toHaveLength(1);
	expect(concatenated[0].modules.map((m) => m.name).sort()).toEqual([
		"./asi.js",
		"./bump.js",
		"./cjs-namespace.js",
		"./counter.js",
		"./destructuring.js",
		"./helper.js",
		"./index.js",
		"./logical.js",
		"./loops.js",
		"./namespace-binding.js",
		"./namespace.js",
		"./quoted-writer.js",
		"./quoted.js",
		"./state.js",
		"./writer.js"
	]);
});
