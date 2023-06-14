import { pathToFileURL } from "url";
import isEsmFile from "./isEsmFile";

const crossImport = async <T = any>(
	path: string,
	cwd = process.cwd()
): Promise<T> => {
	if (isEsmFile(path, cwd)) {
		const url = pathToFileURL(path).href;
		const { default: config } = await import(url);
		return config;
	} else {
		return require(path);
	}
};

export default crossImport;
