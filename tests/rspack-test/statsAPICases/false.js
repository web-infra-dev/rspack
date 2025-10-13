/** @type {import('@rspack/test-tools').TStatsAPICaseConfig} */
module.exports = {
  description: "should omit all properties with all false",
  options(context) {
    return {
      context: context.getSource(),
      entry: "./fixtures/a"
    };
  },
  async check(stats) {
    expect(stats.toJson({
      all: false
    })).toEqual({});
  }
};