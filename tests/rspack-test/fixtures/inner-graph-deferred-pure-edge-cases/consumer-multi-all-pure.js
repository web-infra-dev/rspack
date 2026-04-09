import { pureAllA } from "./dep-all-pure-a";
import { pureAllB } from "./dep-all-pure-b";

const unusedAllPure = pureAllA() + pureAllB();
export const usedAllPure = "all-pure";
