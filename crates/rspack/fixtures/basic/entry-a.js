import { a } from './a';
import { shared } from './shared';

console.log(a, shared);

import('./asynced').then(console.log);
