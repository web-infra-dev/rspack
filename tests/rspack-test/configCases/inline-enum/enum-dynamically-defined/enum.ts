export enum Kind {
  A,
  B,
}

export namespace Kind {
  export const isA = (value: Kind) => value == Kind.A
}

Kind.isB = (value: Kind) => value == Kind.B
