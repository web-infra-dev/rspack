new Promise(function (resolve, reject) {
    try {
      var { readFile } = require('fs');
      var { join } = require('path');
  
      readFile(join(__dirname, $PATH), function(err, buffer){
        if (err) return reject(err);
  
        // Fake fetch response
        resolve({
          arrayBuffer() { return buffer; }
        });
      });
    } catch (err) { reject(err); }
});