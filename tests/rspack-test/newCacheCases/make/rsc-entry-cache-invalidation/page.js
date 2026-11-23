import { A } from './a';
import { B } from './b';
export default [A, B];
---
import { B } from './b';
import { A } from './a';
export default [A, B];
