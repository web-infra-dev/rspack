import { NO_INLINE } from "./lib.ts";

import "./foo?22";

console.log(NO_INLINE);

export const getGeneratedSource = () =>
	require("fs").readFileSync(__filename, "utf-8");
