import { usedDirect } from "./consumer-direct";
import { usedReexport } from "./consumer-reexport-alias";
import { usedStar } from "./consumer-star-reexport";
import { usedAllPure } from "./consumer-multi-all-pure";
import { usedMixed } from "./consumer-multi-mixed";

console.log(usedDirect, usedReexport, usedStar, usedAllPure, usedMixed);
