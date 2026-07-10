import { required } from "./only-require.js";
import { resolved } from "./with-resolve.js";
import {
	inlineUnknownMember,
	unknownMember
} from "./with-unknown-member.js";
import { escapedRequire } from "./with-value-escape.js";

export {
	escapedRequire,
	inlineUnknownMember,
	required,
	resolved,
	unknownMember
};

it("strips handled uses and preserves runtime created requires in CommonJS output", () => {
	expect(required).toBe("dep");
	expect(resolved).toBe("./dep.js");
	expect(unknownMember).toBe(undefined);
	expect(inlineUnknownMember).toBe(undefined);
	expect(typeof escapedRequire).toBe("function");
	expect(escapedRequire.resolve("path")).toBe("path");
});
