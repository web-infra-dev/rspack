jest.setTimeout(180000);

if (!expect.getState().testPath.includes("colors.test.js")) {
  process.env.FORCE_COLOR = 0;
  process.env.NO_COLOR = true;
}
