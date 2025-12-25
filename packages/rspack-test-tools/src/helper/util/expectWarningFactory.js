// @ts-nocheck
module.exports = () => {
  const warnings = [];
  let oldWarn;

  beforeEach(() => {
    oldWarn = console.warn;
    console.warn = (m) => warnings.push(m);
  });

  afterEach(() => {
    expectWarning();
    console.warn = oldWarn;
  });

  const expectWarning = (...regexp) => {
    expect(warnings).toEqual(regexp.map((r) => expect.stringMatching(r)));
    warnings.length = 0;
  };

  return expectWarning;
};
