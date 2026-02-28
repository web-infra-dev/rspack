/** @type {import('@rspack/test-tools').TStatsAPICaseConfig} */
module.exports = {
  description: "should print env string in stats",
  options(context) {
    return {
      context: context.getSource(),
      entry: "./fixtures/a"
    };
  },
  async check(stats) {
    expect(
      stats.toString({
        all: false,
        env: true,
        _env: "production"
      })
    ).toBe('Environment (--env): "production"');
    expect(
      stats.toString({
        all: false,
        env: true,
        _env: {
          prod: ["foo", "bar"],
          baz: true
        }
      })
    ).toBe(
      "Environment (--env): {\n" +
      '  "prod": [\n' +
      '    "foo",\n' +
      '    "bar"\n' +
      "  ],\n" +
      '  "baz": true\n' +
      "}"
    );
  }
};