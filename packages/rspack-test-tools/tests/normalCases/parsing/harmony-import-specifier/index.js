import { Texture as e } from "./a.js";

it("should not parse when logical and with `false && unknown = false`", function () {
  var index_es_e = {};
  var index_es_Ce = {
    get exports() {
		  return index_es_e;
    },
    set exports(e) {
    	index_es_e = e;
    }
  };
  console.log.bind(null, e);
  index_es_Ce.exports = 1;
  expect(index_es_Ce.exports).toBe(1)
});