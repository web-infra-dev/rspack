module.exports = function(source) {
  const logger = this.getLogger('TestLoader');
  logger.group('group')
  logger.info('info something');
  logger.log('log something');
  logger.groupEnd();
  logger.clear();
  return source
}