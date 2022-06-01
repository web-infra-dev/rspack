import { equal } from 'assert';

import("./module").then((res) => {
  equal(res, "module");
})