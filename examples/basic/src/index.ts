class Parent {
	time: undefined | number = new Date().valueOf();
}

class Child extends Parent {
	time: undefined | number;
}