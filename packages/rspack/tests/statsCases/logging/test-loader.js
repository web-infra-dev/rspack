module.exports = function(source) {
  const logger = this.getLogger('TestLoader');
  logger.info('info something');
  logger.log('log something');
  return source
}