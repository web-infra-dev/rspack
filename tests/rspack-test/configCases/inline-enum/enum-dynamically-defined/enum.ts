export enum Kind {
  A,
  B,
}

export namespace Kind {
  export const isA = (value: Kind) => value == Kind.A
  export const aaa = "aaa"
}

Kind.isB = (value: Kind) => value == Kind.B
