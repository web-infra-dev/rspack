import { foo } from './a.js'
class Derived extends Object {
  [foo]() {
    super[foo]();
  }
}

const instance = new Derived();
instance[foo]();