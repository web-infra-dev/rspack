import { readCyclicConst } from "./cycle-b";

export const cyclicConst = "cyclic";

export function readFromCycle() {
	return readCyclicConst();
}
