export interface QualityInspection {
  title?: string;
  time?: string;
  qualityCount?: number;
  randomCount?: number;
  duration?: number;
}

export interface BasicCard {
  icon?: number;
  status?: 0 | 1 | 2;
  description?: string;
}
