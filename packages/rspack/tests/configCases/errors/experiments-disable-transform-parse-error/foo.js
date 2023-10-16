function clsDecorator(cls) {
	cls.prototype.a = 1;
}
@clsDecorator
class A {}
