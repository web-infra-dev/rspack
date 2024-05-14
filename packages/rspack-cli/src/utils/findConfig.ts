import fs from "fs";

import { DEFAULT_EXTENSIONS } from "../constants";

/**
 * Takes a basePath like `webpack.config`, return `webpack.config.{ext}` if
 * exists. returns undefined if none of them exists
 */
const findConfig = (basePath: string): string | undefined => {
	return DEFAULT_EXTENSIONS.map(ext => basePath + ext).find(fs.existsSync);
};

export default findConfig;
