import { required } from "./only-require.js";
import { resolved } from "./with-resolve.js";
import { unknownMember } from "./with-unknown-member.js";
import {
	mixedRequired,
	mixedResolved,
	mixedUnknownMember
} from "./mixed-parsed-and-preserved.js";
import { escapedRequireType } from "./with-value-escape.js";

export {
	escapedRequireType,
	mixedRequired,
	mixedResolved,
	mixedUnknownMember,
	required,
	resolved,
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
});
