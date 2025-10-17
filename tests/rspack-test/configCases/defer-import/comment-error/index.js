import defer { f } from "./mod.js"; // error
import defer f2 from "./mod.js"; // error
import defer * as f3 from "./mod.js";
import defer f4, { f as f5 } from "./mod.js"; // error

export defer * as f4 from "./mod.js"; // error
export defer { f as f5 } from "./mod.js"; // error

export default [f, f2, f3, f4, f5];

export { f3 }
