interface InternalBase {
  id: string;
}

export interface Shared extends InternalBase {
  ready: boolean;
}

export interface UnusedShared {
  dropped: true;
}
