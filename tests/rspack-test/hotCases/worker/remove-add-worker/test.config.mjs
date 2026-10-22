export default {
  moduleScope(ms) {
    ms._globalAssign = {
      ...ms._globalAssign,
      Worker: ms.Worker,
    };
  }
};
