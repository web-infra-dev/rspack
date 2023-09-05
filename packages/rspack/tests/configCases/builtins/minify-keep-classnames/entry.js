class KeepClass {
	constructor() {
		this.name = "test-keep-class-names";
	}

	getName() {
		return this.name;
	}
}

const keepClass = new KeepClass();

keepClass.getName();
