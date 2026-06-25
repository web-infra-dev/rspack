import { test as onlyA } from "./only-a";
import { test as onlyB } from "./only-b";

const generated = /** @type {string} */ (require("fs").readFileSync(__filename, "utf-8"));

onlyA(it, generated);
onlyB(it, generated);
