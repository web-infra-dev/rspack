import { foo } from './reexport';foo;
export default foo;
---
import { foo, bar } from './reexport';
export default foo + bar;
