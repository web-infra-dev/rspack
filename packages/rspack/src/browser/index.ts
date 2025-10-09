export * from "../index";
export { BrowserHttpImportEsmPlugin } from "./BrowserHttpImportEsmPlugin";
export { BrowserRequirePlugin } from "./BrowserRequirePlugin";

import { fs, volume } from "./fs";
export const builtinMemFs = {
	fs,
	volume
};
