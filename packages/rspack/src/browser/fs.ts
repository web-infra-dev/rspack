// @ts-ignore: shimmed with rspack.wasi-browser.js
import { __fs, __volume } from "@rspack/binding";
import type { IFs, Volume } from "memfs";
export const fs: IFs = __fs;
export const volume: Volume = __volume;

const { readFileSync, readdirSync, lstat, existsSync, readdir, watch } = fs;

export default fs;
export { readFileSync, readdirSync, lstat, existsSync, readdir, watch };
