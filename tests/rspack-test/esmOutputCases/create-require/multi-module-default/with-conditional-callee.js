import { createRequire } from "node:module";

const require = (true ? createRequire : null)(import.meta.url);

export const conditionalUnknownMember = require.a;
