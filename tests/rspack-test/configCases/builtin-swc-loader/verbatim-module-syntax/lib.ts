import type { User, UserRole } from "./types";
import { DEFAULT_ROLE } from "./types";

export const getDefaultUser = (): User => {
  return {
    name: "test",
    age: 18
  };
};

export const defaultRole: UserRole = DEFAULT_ROLE;

