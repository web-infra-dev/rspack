import { createRequire } from "node:module";

const require = createRequire(import.meta.url);
export const directRequire = createRequire(import.meta.url);
const createRequireAlias = createRequire;
const aliasedCalleeRequire = createRequireAlias(import.meta.url);
const copiedRequire = require;
let assignedRequire;
assignedRequire = require;

export default require;
export {
	aliasedCalleeRequire,
	assignedRequire,
	copiedRequire,
	require as exportedRequire
};
