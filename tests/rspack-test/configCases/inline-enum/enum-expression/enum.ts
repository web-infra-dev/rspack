export enum E {
  Dynamic = Math.random(),
  Static = 1,
}

export enum Evaluatable {
  Neg = -1,
  Pos = +1,
  Add = 1 + 1,
  Ref = Pos * 3,
  Ref2 = Evaluatable.Pos * 4,
  Tail,
}
