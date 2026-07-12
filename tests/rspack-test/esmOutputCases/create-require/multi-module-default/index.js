import { required } from "./only-require.js";
import { resolved } from "./with-resolve.js";
import { unknownMember } from "./with-unknown-member.js";
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
	beforeAliasedCalleeUnknownMember,
	beforeAssignmentUnknownMember,
	beforeDeclarationBuiltinJoinType,
	beforeDeclarationUnknownMember,
	handledAliasedCalleeRequired,
	preInitializationAliasThrows,
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
	beforeAssignmentUnknownMember,
	beforeDeclarationBuiltinJoinType,
	beforeDeclarationUnknownMember,
	conditionalUnknownMember,
	handledAliasedCalleeRequired,
	inlineEscapedRequireType,
	ignoredRequiredJoinType,
	ignoredResolved,
	inlineIgnoredRequiredJoinType,
	inlineIgnoredResolved,
	mixedRequired,
	mixedResolved,
	mixedUnknownMember,
	mutationUnknowns,
	nonDeferredEscapedRequireType,
	nonDeferredUnknownMember,
	preInitializationAliasThrows,
	preInitializationThrows,
	required,
	resolved,
	shadowedCalleeThrows,
	unknownMember
};

it("preserves created require only for unhandled usages across modules", () => {
	expect(required).toBe("dep");
	expect(resolved).toBe("./dep.js");
	expect(unknownMember).toBe(undefined);
	expect(mixedRequired).toBe("dep");
	expect(mixedResolved).toBe("./dep.js");
	expect(mixedUnknownMember).toBe(undefined);
	expect(escapedRequireType).toBe("function");
	expect(inlineEscapedRequireType).toBe("function");
	expect(nonDeferredEscapedRequireType).toBe("function");
	expect(nonDeferredUnknownMember).toBe(undefined);
	expect(conditionalUnknownMember).toBe(undefined);
	expect(beforeAliasedCalleeUnknownMember).toBe(undefined);
	expect(beforeAssignmentUnknownMember).toBe(undefined);
	expect(beforeDeclarationBuiltinJoinType).toBe("function");
	expect(beforeDeclarationUnknownMember).toBe(undefined);
	expect(handledAliasedCalleeRequired).toBe("dep");
	expect(preInitializationAliasThrows).toEqual([true, true]);
	expect(preInitializationThrows).toEqual([true, true, true]);
	expect(shadowedCalleeThrows).toBe(true);
	expect(ignoredRequiredJoinType).toBe("function");
	expect(ignoredResolved).toBe("path");
	expect(inlineIgnoredRequiredJoinType).toBe("function");
	expect(inlineIgnoredResolved).toBe("path");
	expect(mutationUnknowns).toEqual([
		"function",
		"function",
		undefined,
		undefined
	]);
});
