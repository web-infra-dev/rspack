import { destroyMaybe as i } from "./util.js";
class j {
	constructor() {
    // should automatically insert semicolon
		this.ground = 1
	}
	destroy() {
		i(),
		i();
		return 1;
	}
}

it("methods after the ctor should not insert unexpected semicolon", () => {
  expect(new j().destroy()).toBe(1);
});

