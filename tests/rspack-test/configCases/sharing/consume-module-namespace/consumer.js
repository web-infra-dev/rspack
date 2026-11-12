import cjsDefault, { greet } from "./cjs.js";
import * as cjsNamespace from "./cjs.js";
import esmDefault, { named } from "./esm.mjs";
import * as esmNamespace from "./esm.mjs";

export const values = {
	cjsDefault,
	cjsNamespace,
	greet,
	esmDefault,
	esmNamespace,
	named
};
