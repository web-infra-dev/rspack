// Helper functions for the fake node module

exports.deepClone = function(obj) {
  if (obj === null || typeof obj !== 'object') {
    return obj;
  }
  
  if (obj instanceof Date) {
    return new Date(obj);
  }
  
  if (obj instanceof Array) {
    return obj.map(item => exports.deepClone(item));
  }
  
  const cloned = {};
  Object.keys(obj).forEach(key => {
    cloned[key] = exports.deepClone(obj[key]);
  });
  
  return cloned;
};

exports.mergeObjects = function(target, ...sources) {
  if (!target) return {};
  
  sources.forEach(source => {
    if (source) {
      Object.keys(source).forEach(key => {
        if (typeof source[key] === 'object' && source[key] !== null && !Array.isArray(source[key])) {
          target[key] = exports.mergeObjects(target[key] || {}, source[key]);
        } else {
          target[key] = source[key];
        }
      });
    }
  });
  
  return target;
};

exports.arrayToObject = function(arr, keyField) {
  return arr.reduce((obj, item) => {
    const key = typeof keyField === 'function' ? keyField(item) : item[keyField];
    obj[key] = item;
    return obj;
  }, {});
};

exports.groupBy = function(arr, keyField) {
  return arr.reduce((groups, item) => {
    const key = typeof keyField === 'function' ? keyField(item) : item[keyField];
    if (!groups[key]) {
      groups[key] = [];
    }
    groups[key].push(item);
    return groups;
  }, {});
};

// Unused exports
exports.unusedHelper = function() {
  return "unused helper function";
};