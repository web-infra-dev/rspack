import { foo, Base } from "./a.js";
class Derived extends Base {
	[foo]() {
		super[foo](); // <-- ERROR HERE
	}
}

const instance = new Derived();
instance[foo]();
