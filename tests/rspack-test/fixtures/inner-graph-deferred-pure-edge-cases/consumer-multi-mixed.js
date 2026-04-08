import { pureMixedA } from "./dep-mixed-a";
import { impureMixedB } from "./dep-mixed-b";

const unusedMixed = pureMixedA() + impureMixedB();
export const usedMixed = "mixed";
