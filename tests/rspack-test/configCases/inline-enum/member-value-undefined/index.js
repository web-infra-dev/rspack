import { test as onlyA } from "./only-a";
import { test as onlyB } from "./only-b";

const generated = /** @type {string} */ (__non_webpack_require__("fs").readFileSync(__filename, "utf-8"));

onlyA(it, generated);
onlyB(it, generated);
