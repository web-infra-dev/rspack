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

export {
	cacheType,
	escapedRequireType,
	inlineResolved,
	mixedDisabledRequired,
	mixedDisabledResolved,
	mixedDisabledUnknownMember,
	required,
	resolved,
	resolvePaths,
	resolveWithOptions,
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
});
