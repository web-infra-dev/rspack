/*
 * Test fails: Expected error not emitted
 * Should emit error if entry point and splitted chunk have the same name
 */
export default () => 'should emit error if entry point and splitted chunk have the same name https://github.com/web-infra-dev/rspack/issues/4332'
