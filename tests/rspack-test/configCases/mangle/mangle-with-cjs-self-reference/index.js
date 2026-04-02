it("should correctly mangle exports with cjs self reference (full require)", () => {
    const a = require("./a?fullrequire").getLongLongLong;
    const aLongLongLongCanMangle = require("./a?fullrequire").LongLongLongCanMangle;
    const aGetLongLongLongCanMangle = require("./a?fullrequire").getLongLongLongCanMangle;
    const b = require("./b?fullrequire").getLongLongLong;
    const bLongLongLongCanMangle = require("./b?fullrequire").LongLongLongCanMangle;
    const bGetLongLongLongCanMangle = require("./b?fullrequire").getLongLongLongCanMangle;
    const c = require("./c?fullrequire").getLongLongLong();
    const cLongLongLongCanMangle = require("./c?fullrequire").LongLongLongCanMangle;
    const cGetLongLongLongCanMangle = require("./c?fullrequire").getLongLongLongCanMangle;
    expect(a).toBe("a");
    expect(aLongLongLongCanMangle).toBe(true);
    expect(aGetLongLongLongCanMangle).toBe(true);
    expect(b).toBe("b");
    expect(bLongLongLongCanMangle).toBe(true);
    expect(bGetLongLongLongCanMangle).toBe(true);
    expect(c).toBe("c");
    expect(cLongLongLongCanMangle).toBe(true);
    expect(cGetLongLongLongCanMangle).toBe(true);
});

it("should correctly mangle exports with cjs self reference (require variable)", () => {
    const a = require("./a?requirevariable");
    const aValue = a.getLongLongLong;
    const aLongLongLongCanMangle = a.LongLongLongCanMangle;
    const aGetLongLongLongCanMangle = a.getLongLongLongCanMangle;
    const b = require("./b?requirevariable");
    const bValue = b.getLongLongLong;
    const bLongLongLongCanMangle = b.LongLongLongCanMangle;
    const bGetLongLongLongCanMangle = b.getLongLongLongCanMangle;
    const c = require("./c?requirevariable");
    const cValue = c.getLongLongLong();
    const cLongLongLongCanMangle = c.LongLongLongCanMangle;
    const cGetLongLongLongCanMangle = c.getLongLongLongCanMangle;
    expect(aValue).toBe("a");
    expect(aLongLongLongCanMangle).toBe(true);
    expect(aGetLongLongLongCanMangle).toBe(false);
    expect(bValue).toBe("b");
    expect(bLongLongLongCanMangle).toBe(true);
    expect(bGetLongLongLongCanMangle).toBe(false);
    expect(cValue).toBe("c");
    expect(cLongLongLongCanMangle).toBe(true);
    expect(cGetLongLongLongCanMangle).toBe(false);
});

it("should correctly mangle exports with cjs self reference (require destructuring)", () => {
    const {
        getLongLongLong: aValue,
        LongLongLongCanMangle: aLongLongLongCanMangle,
        getLongLongLongCanMangle: aGetLongLongLongCanMangle,
    } = require("./a?requiredestructuring");
    const {
        getLongLongLong: bValue,
        LongLongLongCanMangle: bLongLongLongCanMangle,
        getLongLongLongCanMangle: bGetLongLongLongCanMangle,
    } = require("./b?requiredestructuring");
    const {
        getLongLongLong: cValue,
        LongLongLongCanMangle: cLongLongLongCanMangle,
        getLongLongLongCanMangle: cGetLongLongLongCanMangle,
    } = require("./c?requiredestructuring");
    expect(aValue).toBe("a");
    expect(aLongLongLongCanMangle).toBe(true);
    expect(aGetLongLongLongCanMangle).toBe(false);
    expect(bValue).toBe("b");
    expect(bLongLongLongCanMangle).toBe(true);
    expect(bGetLongLongLongCanMangle).toBe(false);
    expect(cValue()).toBe("c");
    expect(cLongLongLongCanMangle).toBe(true);
    expect(cGetLongLongLongCanMangle).toBe(false);
});
