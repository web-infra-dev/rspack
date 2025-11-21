export function test() {}

export class Cls {
	constructor() {}
}

if (__AAA__) {
	const __Cls = Cls;
	__Cls.prototype.aaa = function (a, b) {};
}
