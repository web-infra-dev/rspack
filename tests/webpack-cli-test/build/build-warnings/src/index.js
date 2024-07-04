let obj;

try {
  obj = require('unknown');  
} catch (e) {
    // Ignore
}

export default obj
