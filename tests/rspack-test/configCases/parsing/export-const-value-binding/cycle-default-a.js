import { readCyclicDefault } from "./cycle-default-b";

const cyclicDefault = "cyclic-default";

export default cyclicDefault;

export function readFromDefaultCycle() {
	return readCyclicDefault();
}
