/* eslint-disable import/no-namespace */
import * as a from "./a.css";
import * as b from "./b.css";

import * as all from "./index";

export * from "./c.css";
export { a, b };

// eslint-disable-next-line no-console
console.log(all);
