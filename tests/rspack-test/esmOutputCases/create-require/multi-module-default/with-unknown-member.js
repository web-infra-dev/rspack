import { createRequire } from "node:module";

const require = createRequire(import.meta.url);
const optionalCacheRequire = createRequire(import.meta.url);

export const unknownMember = require.a;
export const unknownMemberType = typeof require.a;
export const optionalCacheCallThrows = (() => {
	try {
		optionalCacheRequire?.cache();
		return false;
	} catch {
		return true;
	}
})();
