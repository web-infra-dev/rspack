import fs from 'fs'
import value, * as utils from './proxy.js'

it('should not contain duplicate initFragment for namespace_cache', () => {
  const { foo } = utils;
  access(utils)
  access(utils.bar)
  access(value)
  access(value.bar)
  access(foo)

  expect([...fs.readFileSync(__filename).toString().matchAll(/var _proxy_js__WEBPACK_IMPORTED_MODULE_0___namespace_cache/g)].length).toBe(2);
})

function access(a) {
  a
}
