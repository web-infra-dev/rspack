import { required } from "./only-require.js";
import { resolved } from "./with-resolve.js";
import { inlineResolved } from "./inline-resolve.js";
import { cacheType } from "./with-cache.js";
import { unknownMember } from "./with-unknown-member.js";
import { resolvePaths } from "./with-resolve-paths.js";
import {
	mixedDisabledRequired,
	mixedDisabledResolved,
	mixedDisabledUnknownMember
} from "./mixed-parsed-and-preserved.js";
import { resolveWithOptions } from "./with-resolve-options.js";
import { escapedRequireType } from "./with-value-escape.js";
import { sequenceResolved } from "./with-sequence-callee.js";
import {
	extraArgEffects,
	extraArgRequired,
	inlineExtraArgResolved
} from "./with-extra-create-require-args.js";
import { conditionalCopyResolved } from "./with-conditional-copy.js";

export {
	cacheType,
	conditionalCopyResolved,
	escapedRequireType,
	extraArgEffects,
	extraArgRequired,
	inlineResolved,
	inlineExtraArgResolved,
	mixedDisabledRequired,
	mixedDisabledResolved,
	mixedDisabledUnknownMember,
	required,
	resolved,
	resolvePaths,
	resolveWithOptions,
	sequenceResolved,
	unknownMember
};

it("preserves only modules with runtime created-require usages when requireResolve is disabled", () => {
	expect(required).toBe("dep");
	expect(resolved).toBe("path");
	expect(inlineResolved).toBe("path");
	expect(resolvePaths).toBe(null);
	expect(unknownMember).toBe(undefined);
	expect(cacheType).toBe("object");
	expect(mixedDisabledRequired).toBe("dep");
	expect(mixedDisabledResolved).toBe("path");
	expect(mixedDisabledUnknownMember).toBe(undefined);
	expect(resolveWithOptions).toBe("path");
	expect(escapedRequireType).toBe("function");
	expect(sequenceResolved).toBe("path");
	expect(extraArgRequired).toBe("dep");
	expect(inlineExtraArgResolved).toBe("path");
	expect(extraArgEffects).toEqual([true, true]);
	expect(conditionalCopyResolved).toEqual(["path", "path"]);
});
