export interface User {
  name: string;
  age: number;
}

export type UserRole = "admin" | "user";

export const DEFAULT_ROLE: UserRole = "user";

