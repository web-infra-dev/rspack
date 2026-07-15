import * as moduleNamespace from "node:module";

const namespaceRequire = moduleNamespace.createRequire(import.meta.url);

export const namespaceUnknown = namespaceRequire.unknown;
