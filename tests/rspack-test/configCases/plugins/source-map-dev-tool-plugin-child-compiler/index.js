it("should successfully compile and retrieve assets from the child compiler", function () {
    const child = require('./child');
    expect(child.foo).toBe(1);
});
