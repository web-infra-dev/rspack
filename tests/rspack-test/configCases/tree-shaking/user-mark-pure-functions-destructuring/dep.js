const lib = {
  fromObject: () => { unreachable(); },
};
const tuple = [() => { unreachable(); }];

const { fromObject } = lib;
const [fromArray] = tuple;

fromObject();
fromArray();
