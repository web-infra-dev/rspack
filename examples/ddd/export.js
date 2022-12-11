import {defaults} from './Layout'
class Test {
  test = defaults.test + 20000;
}
export var Something = 333;


// export default {
//   a: new Test()
// }
export default function() {
  new Test();
}