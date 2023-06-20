module.exports =  {
  "forceExit": true,
  "setupFiles": [
    "../../setupEnv.js"
  ],
  "setupFilesAfterEnv": [
    "../../setupTestFramework.js"
  ],
  "testMatch": [
    "**/*.test.js",
  ],
  "testEnvironment": "../../patch-node-env.js"
}
