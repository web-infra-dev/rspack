import * as utils from './proxy.js'
import value from './proxy.js'

import fs from 'fs'

it('should not contain duplicate initFragment for namespace_cache', () => {
  const { foo } = utils;
  access(utils)
  access(utils.bar)
  access(value)
  access(value.bar)
  access(foo)

  expect([...fs.readFileSync(__filename).toString().matchAll(/var _proxy_js__rspack_import_0_namespace_cache/g)].length).toBe(2);
})

function access(a) {
  a
}
