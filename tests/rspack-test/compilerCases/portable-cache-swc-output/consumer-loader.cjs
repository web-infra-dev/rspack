module.exports = function(source, map, data) {
  if (!data || data.preserved !== 'producer-data' || data.resource !== this.resourcePath) {
    throw new Error('SWC did not preserve producer additional data');
  }
  this.experiments.portableOutput({ paths: [this.resourcePath] });
  this.callback(null, source + '\nexport const metadata = ' + JSON.stringify(data) + ';', map, data);
};
module.exports.pitch = function() { this.experiments.portableOutput(); };
