module.exports = {
  findBundle(index) {
    switch (index) {
      case 0: return ['resource.js'];
      case 1: return ['request.js'];
      case 2: return ['duplicate.js'];
    }
  }
};