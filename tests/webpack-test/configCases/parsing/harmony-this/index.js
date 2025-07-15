"use strict";


import * as abc from "./abc";
import d, {C as _C, D as _D, a, b as B, E, extendThisClass, F, f1, f2, f3, G, returnThisArrow, returnThisMember, that} from "./abc";
import {applyThis, bindThis, callThis } from "./issue-7213";

it("should have this = undefined on ES modules", () => {
	expect((typeof that)).toBe("undefined");
	expect((typeof abc.that)).toBe("undefined");
	expect((typeof returnThisArrow())).toBe("undefined");
	expect((typeof abc.returnThisArrow())).toBe("undefined");
	expect(function() {
		returnThisMember();
	}).toThrowError();
	expect(function() {
		abc.returnThisMember();
	}).toThrowError();
	expect(function() {
		extendThisClass();
	}).toThrowError();
});

it("should not break classes and functions", () => {
	expect((new _C).foo()).toBe("bar");
	expect((new _C).bar()).toBe("bar");
	expect((new _D).prop()).toBe("ok");
	expect(E.foo()).toBe("bar");
	expect(F).toBe("ok");
	expect(f1.call({x: "f1"})).toBe("f1");
	expect(f2.call({x: "f2"})).toBe("f2");
	expect(f3.call("f3")).toBe(undefined);
	expect(f3()).toBe(undefined);
	expect((new G("ok")).getX()).toBe("ok");
});

function x() {
	throw new Error("should not be executed");
}

it("should have this = undefined on imported non-strict functions", () => {
	x
	d
	x
	d()
	expect(d()).toBe("undefined");
	x
	a
	x
	a()
	expect(a()).toBe("undefined");
	x
	B
	x
	B()
	expect(B()).toBe("undefined");
	x
	abc.a
	x
	abc.a()
	expect(abc.a()).toBeTypeOf("object");
	var thing = abc.a();
	expect(Object.keys(thing)).toEqual(Object.keys(abc));
});


import * as New from "./new";
import C2, { C } from "./new";

it("should be possible to use new correctly", () => {
	expect(new C()).toEqual({ok: true});
	expect(new C2()).toEqual({ok: true});
	expect(new New.C()).toEqual({ok: true});
});

it("should not break Babel arrow function transform", () => {
	expect(bindThis()).toBe(undefined);
	expect(callThis).toBe(undefined);
	expect(applyThis).toBe(undefined);
});
