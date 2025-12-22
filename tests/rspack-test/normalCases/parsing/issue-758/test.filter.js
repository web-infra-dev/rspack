/*
 * Test fails with error:
 * Warnings while compiling:
 * Module not found: Can't resolve './missingModule'
 * The test expects warnings about missing modules but they cause test failure
 */

module.exports = () => "TODO: https://github.com/web-infra-dev/rspack/issues/4313"

