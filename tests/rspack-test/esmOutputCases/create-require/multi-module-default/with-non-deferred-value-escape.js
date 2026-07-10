import { createRequire } from "node:module";

function identity(value) {
	return value;
}

const require = createRequire(new URL(import.meta.url));

export const nonDeferredEscapedRequireType = typeof identity(require);
export const nonDeferredUnknownMember = require.a;
