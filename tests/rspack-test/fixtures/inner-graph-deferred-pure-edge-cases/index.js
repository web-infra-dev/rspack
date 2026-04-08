import { usedDirect } from "./consumer-direct";
import { usedDirectAlias } from "./consumer-direct-alias";
import { usedDefaultAlias } from "./consumer-default-alias";
import { usedReexport } from "./consumer-reexport-alias";
import { usedStar } from "./consumer-star-reexport";
import { usedAllPure } from "./consumer-multi-all-pure";
import { usedMixed } from "./consumer-multi-mixed";

console.log(
	usedDirect,
	usedDirectAlias,
	usedDefaultAlias,
	usedReexport,
	usedStar,
	usedAllPure,
	usedMixed
);
