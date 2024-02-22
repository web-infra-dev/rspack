import fs from "fs";

export const isDirectory = (p: string) => fs.lstatSync(p).isDirectory();
export const isValidCaseDirectory = (name: string) =>
	!name.startsWith("_") && !name.startsWith(".");
