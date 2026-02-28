function EventEmitter() {
  this.events = {};
}

EventEmitter.prototype.on = function (eventName, callback) {
  if (!this.events[eventName]) {
    this.events[eventName] = [];
  }
  this.events[eventName].push(callback);
};

EventEmitter.prototype.emit = function (eventName) {
  var args = Array.prototype.slice.call(arguments, 1);
  if (this.events[eventName]) {
    this.events[eventName].forEach(function (callback) {
      callback.apply(null, args);
    });
  }
};

var emitter = new EventEmitter();

// TODO: remove default export when rspack-dev-server refactored
export default emitter;
export { emitter };
