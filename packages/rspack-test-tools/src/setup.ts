import {
	defineCompileCase,
	defineDefaultsCase,
	defineErrorCase,
	defineStatsAPICase
} from "./case";
import { basename, casename } from "./helper";

const Utils = {
	basename,
	casename
};

(globalThis as any).defineCompileCase = defineCompileCase;
(globalThis as any).defineErrorCase = defineErrorCase;
(globalThis as any).defineStatsAPICase = defineStatsAPICase;
(globalThis as any).defineDefaultsCase = defineDefaultsCase;
(globalThis as any).Utils = Utils;
