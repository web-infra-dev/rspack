import { createRequire } from "node:module";

const require = createRequire(import.meta.url);

export const unknownMember = require.a;
export const inlineUnknownMember = createRequire(import.meta.url).a;
