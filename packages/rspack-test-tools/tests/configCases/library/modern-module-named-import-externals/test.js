// re export
export { a } from 'externals0'

// named import
import { a as a_2 } from 'externals1'

// default import
import defaultValue from 'externals2'

// namespace import
import * as namespace from 'externals3'

// side effect only import
import 'externals4'

import './lib'

a_2;
defaultValue;
namespace;
