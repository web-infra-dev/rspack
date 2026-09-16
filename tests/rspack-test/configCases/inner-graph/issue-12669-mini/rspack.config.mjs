import createTestCases from '../_helpers/createTestCases.js';
export default createTestCases({
  // nothing: {
  // 	usedExports: [],
  // 	expect: {
  // 		"./dependency": []
  // 	}
  // },
  a: {
    usedExports: ['a'],
    expect: {
      './dependency': ['x'],
    },
  },
  // b: {
  // 	usedExports: ["b"],
  // 	expect: {
  // 		"./dependency": ["y"]
  // 	}
  // }
});
