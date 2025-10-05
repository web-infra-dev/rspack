module.exports = function(params) {
  console.log('Runtime plugin with params loaded');
  console.log('Received parameters:', params);

  const hasValidParams =
    params &&
    params.testParam1 === 'value1' &&
    params.testParam2 === 123 &&
    params.testParam3 === true;

  console.log('Parameters validation:', hasValidParams ? 'PASS' : 'FAIL');

  return {
    name: 'parametric-plugin',
    params: params,
    getParam: function(key) {
      return params[key];
    },
    validateParams: function() {
      return hasValidParams;
    }
  };
};
