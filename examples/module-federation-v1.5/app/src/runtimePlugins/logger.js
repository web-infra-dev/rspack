export default function () {
  return {
    name: 'logger',
    beforeInit(args) {
      console.log('beforeInit: ', args);
      return args;
    },
    beforeLoadShare(args) {
      console.log('beforeLoadShare: ', args);
      return args;
    },
  };
}
