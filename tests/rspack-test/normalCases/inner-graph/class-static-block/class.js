import { a, b, c, d } from "./module"

let _A;
class A {
  static {
    _A = A;
  }
  prop = a();
}

let _B;
class B {
  static {
    _B = this;
  }
  prop = b();
}

let _C;
let __C = class C {
  static {
    _C = this;
  }
  prop = c();
}

let _D;
let __D = class D {
  static {
    _D = D;
  }
  prop = d();
}

export { _A, _B, _C, _D };
