module.exports = function() {
  console.log('Basic runtime plugin loaded');

  return {
    name: 'basic-plugin',
    version: '1.0.0',
    onInit: function(federationInstance) {
      console.log('Basic plugin initialized with federation instance');

      if (typeof federationInstance.extend === 'function') {
        federationInstance.extend({
          basicPluginMethod: function() {
            return 'Method from basic plugin';
          }
        });
      }
    }
  };
};
