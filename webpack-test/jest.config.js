module.exports =  {
    "forceExit": true,
    "setupFiles": [
      "<rootDir>/setupEnv.js"
    ],
    "setupFilesAfterEnv": [
      "<rootDir>/setupTestFramework.js"
    ],
    "testMatch": [
      // "<rootDir>/*.test.js",
      // "<rootDir>/*.basictest.js",
      // "<rootDir>/*.longtest.js",
      // "<rootDir>/*.unittest.js",
      "<rootDir>/TestCasesNormal.basictest.js",
      "<rootDir>/ConfigTestCases.basictest.js"
    ],
    "watchPathIgnorePatterns": [
      "<rootDir>/.git",
      "<rootDir>/node_modules",
      "<rootDir>/js",
      "<rootDir>/browsertest/js",
      "<rootDir>/fixtures/temp-cache-fixture",
      "<rootDir>/fixtures/temp-",
      "<rootDir>/benchmark",
      "<rootDir>/assembly",
      "<rootDir>/tooling",
      "<rootDir>/examples/*/dist",
      "<rootDir>/coverage",
      "<rootDir>/.eslintcache"
    ],
    "modulePathIgnorePatterns": [
      "<rootDir>/.git",
      "<rootDir>/node_modules/webpack/node_modules",
      "<rootDir>/js",
      "<rootDir>/browsertest/js",
      "<rootDir>/fixtures/temp-cache-fixture",
      "<rootDir>/fixtures/temp-",
      "<rootDir>/benchmark",
      "<rootDir>/examples/*/dist",
      "<rootDir>/coverage",
      "<rootDir>/.eslintcache"
    ],
    "transformIgnorePatterns": [
      "<rootDir>"
    ],
    "coverageDirectory": "<rootDir>/coverage",
    "coveragePathIgnorePatterns": [
      "\\.runtime\\.js$",
      "<rootDir>",
      "<rootDir>/schemas",
      "<rootDir>/node_modules"
    ],
    "testEnvironment": "./patch-node-env.js",
    "coverageReporters": [
      "json"
    ]
  }