export * from "../index";
export { BrowserHttpImportEsmPlugin } from "./BrowserHttpImportEsmPlugin";
export { BrowserRequirePlugin } from "./BrowserRequire";

import { fs, volume } from "./fs";
export const builtinMemFs = {
	fs,
	volume
};
