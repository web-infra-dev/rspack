import { required } from "./only-require.js";
import { resolved } from "./with-resolve.js";
import {
	optionalCacheCallThrows,
	unknownMember,
	unknownMemberType
} from "./with-unknown-member.js";
import {
	mixedRequired,
	mixedResolved,
	mixedUnknownMember
} from "./mixed-parsed-and-preserved.js";
import { escapedRequireType } from "./with-value-escape.js";
import { mutationUnknowns } from "./with-non-dominating-mutation.js";
import { inlineEscapedRequireType } from "./with-inline-value-escape.js";
import {
	nonDeferredEscapedRequireType,
	nonDeferredUnknownMember
} from "./with-non-deferred-value-escape.js";
import { conditionalUnknownMember } from "./with-conditional-callee.js";
import {
	nestedExtraArgRequiredJoinType,
	nestedExtraArgResolved
} from "./with-nested-extra-args.js";
import {
	beforeAliasedCalleeUnknownMember,
	beforeAliasedCalleeRequired,
	beforeAssignmentUnknownMember,
	beforeDeclarationBuiltinJoinType,
	beforeDeclarationRequired,
	beforeDeclarationUnknownMember,
	boundBeforeDeclarationRequired,
	handledAliasedCalleeRequired,
	handledBeforeDeclarationRequired,
	mutableAfterDeclarationResult,
	mutableAssignedCreateRequireResult,
	mutableAssignedRequireResult,
	mutableBeforeDeclarationResult,
	mutableCreateRequireAliasResult,
	preInitializationAliasThrows,
	preInitializationImmediateExecutionThrows,
	preInitializationLogicalAssignmentValues,
	preInitializationThrows,
	shadowedCalleeThrows
} from "./with-use-before-declaration.js";
import {
	ignoredRequiredJoinType,
	ignoredResolved,
	inlineIgnoredRequiredJoinType,
	inlineIgnoredResolved
} from "./with-ignored-uses.js";

export {
	escapedRequireType,
	beforeAliasedCalleeUnknownMember,
	beforeAliasedCalleeRequired,
	beforeAssignmentUnknownMember,
	beforeDeclarationBuiltinJoinType,
	beforeDeclarationRequired,
	beforeDeclarationUnknownMember,
	boundBeforeDeclarationRequired,
	conditionalUnknownMember,
	handledAliasedCalleeRequired,
	handledBeforeDeclarationRequired,
	mutableAfterDeclarationResult,
	mutableAssignedCreateRequireResult,
	mutableAssignedRequireResult,
	mutableBeforeDeclarationResult,
	mutableCreateRequireAliasResult,
	inlineEscapedRequireType,
	ignoredRequiredJoinType,
	ignoredResolved,
	inlineIgnoredRequiredJoinType,
	inlineIgnoredResolved,
	mixedRequired,
	mixedResolved,
	mixedUnknownMember,
	mutationUnknowns,
	nestedExtraArgRequiredJoinType,
	nestedExtraArgResolved,
	nonDeferredEscapedRequireType,
	nonDeferredUnknownMember,
	preInitializationAliasThrows,
	preInitializationImmediateExecutionThrows,
	preInitializationLogicalAssignmentValues,
	preInitializationThrows,
	required,
	resolved,
	shadowedCalleeThrows,
	optionalCacheCallThrows,
	unknownMember,
	unknownMemberType
};

it("preserves created require only for unhandled usages across modules", () => {
	expect(required).toBe("dep");
	expect(resolved).toBe("./dep.js");
	expect(unknownMember).toBe(undefined);
	expect(unknownMemberType).toBe("undefined");
	expect(optionalCacheCallThrows).toBe(true);
	expect(mixedRequired).toBe("dep");
	expect(mixedResolved).toBe("./dep.js");
	expect(mixedUnknownMember).toBe(undefined);
	expect(escapedRequireType).toBe("function");
	expect(inlineEscapedRequireType).toBe("function");
	expect(nonDeferredEscapedRequireType).toBe("function");
	expect(nonDeferredUnknownMember).toBe(undefined);
	expect(conditionalUnknownMember).toBe(undefined);
	expect(beforeAliasedCalleeUnknownMember).toBe(undefined);
	expect(beforeAliasedCalleeRequired).toBe("dep");
	expect(beforeAssignmentUnknownMember).toBe(undefined);
	expect(beforeDeclarationBuiltinJoinType).toBe("function");
	expect(boundBeforeDeclarationRequired).toBe("dep");
	expect(beforeDeclarationRequired).toBe("dep");
	expect(beforeDeclarationUnknownMember).toBe(undefined);
	expect(handledAliasedCalleeRequired).toBe("dep");
	expect(handledBeforeDeclarationRequired).toBe("dep");
	expect(mutableAfterDeclarationResult).toBe("./dep.js");
	expect(mutableAssignedCreateRequireResult).toBe("./dep.js");
	expect(mutableAssignedRequireResult).toBe("./dep.js");
	expect(mutableBeforeDeclarationResult).toBe("./dep.js");
	expect(mutableCreateRequireAliasResult).toBe("./dep.js");
	expect(preInitializationAliasThrows).toEqual([true, true]);
	expect(preInitializationImmediateExecutionThrows).toEqual([
		true,
		true,
		true,
		true,
		true,
		true,
		true,
		true
	]);
	expect(preInitializationLogicalAssignmentValues).toEqual(["dep", "dep"]);
	expect(preInitializationThrows).toEqual([true, true, true, true, true]);
	expect(shadowedCalleeThrows).toBe(true);
	expect(ignoredRequiredJoinType).toBe("function");
	expect(ignoredResolved).toBe("path");
	expect(inlineIgnoredRequiredJoinType).toBe("function");
	expect(inlineIgnoredResolved).toBe("path");
	expect(mutationUnknowns).toEqual([
		"function",
		"function",
		undefined,
		undefined,
		undefined
	]);
	expect(nestedExtraArgRequiredJoinType).toBe("function");
	expect(nestedExtraArgResolved).toBe("path");
});
