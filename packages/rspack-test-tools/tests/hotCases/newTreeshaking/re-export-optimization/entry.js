import { foo } from './reexport';foo;
export default foo;
---

import { bar, foo } from './reexport';

export default foo + bar;
