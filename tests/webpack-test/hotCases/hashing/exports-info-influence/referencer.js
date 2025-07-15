export default 42;
---

import { test as value2 } from "external";
import { test as value1 } from "./module";

export default `${value1} ${value2}`;
