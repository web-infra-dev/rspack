import { Texture as e } from "./a.js";

it("should work", function () {
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

it("should work", function () {
  var num;
  var obj = {
    get prop1() {
      var e = 1;
		  return e;
    },
    set prop2(val) {
      var e = val;
    	num = e;
    }
  };
  expect(obj.prop1).toBe(1);
  obj.prop2 = 2;
  expect(num).toBe(2);
});
