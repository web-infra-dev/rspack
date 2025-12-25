/*#__NO_SIDE_EFFECTS__*/ const foo1 = () => { }
/*#__NO_SIDE_EFFECTS__*/ const foo2 = function () { }
/*#__NO_SIDE_EFFECTS__*/ const foo3 = async () => { }
/*#__NO_SIDE_EFFECTS__*/ const foo4 = function* () { }
/*#__NO_SIDE_EFFECTS__*/ const foo5 = async function* () { }

const foo6 = /*#__NO_SIDE_EFFECTS__*/ () => { }
const foo7 = /*#__NO_SIDE_EFFECTS__*/ function () { }
const foo8 = /*#__NO_SIDE_EFFECTS__*/ async () => { }
const foo9 = /*#__NO_SIDE_EFFECTS__*/ async function () { }
const foo10 = /*#__NO_SIDE_EFFECTS__*/ function* () { }

/*#__NO_SIDE_EFFECTS__*/ function foo11() { }
/*#__NO_SIDE_EFFECTS__*/ async function foo12() { }
/*#__NO_SIDE_EFFECTS__*/ function* foo13() { }

export const foo14 = /*#__NO_SIDE_EFFECTS__*/ () => { }
export const foo15 = /*#__NO_SIDE_EFFECTS__*/ function () { }
export const foo16 = /*#__NO_SIDE_EFFECTS__*/ async () => { unreacheable() }
export const foo17 = /*#__NO_SIDE_EFFECTS__*/ async function () { }
export const foo18 = /*#__NO_SIDE_EFFECTS__*/ function* () { }

/*#__NO_SIDE_EFFECTS__*/ export const foo19 = () => { }
/*#__NO_SIDE_EFFECTS__*/ export const foo20 = function () { }
/*#__NO_SIDE_EFFECTS__*/ export const foo21 = async () => { }
/*#__NO_SIDE_EFFECTS__*/ export const foo22 = async function () { }
/*#__NO_SIDE_EFFECTS__*/ export const foo23 = function* () { }
/*#__NO_SIDE_EFFECTS__*/ export const foo24 = async function* () { }

export default /*#__NO_SIDE_EFFECTS__*/ function foo25() { }

foo1();
foo2();
foo3();
foo4();
foo5();
foo6();
foo7();
foo8();
foo9();
foo10();

foo11();
foo12();
foo13();
foo14();
foo15();
foo16();
foo17();
foo18();
foo19();
foo20();
foo21();
foo22();
foo23();
foo24();
foo25();