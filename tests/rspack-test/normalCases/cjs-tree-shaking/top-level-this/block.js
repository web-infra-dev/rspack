if (this.aaa !== 1) {
	this.aaa = 1;
}

while (true) {
	Object.defineProperty(this, "bbb", {
		value: 2
	});
	break;
}
