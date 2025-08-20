export * from "../index";
export { BrowserHttpImportPlugin } from "./BrowserHttpImportPlugin";

import { fs, volume } from "./fs";
export const builtinMemFs = {
	fs,
	volume
};
