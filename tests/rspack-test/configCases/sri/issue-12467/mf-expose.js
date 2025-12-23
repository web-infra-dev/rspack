import { lookup } from "mime-types";

export function mime() {
  return lookup("file.png");
}
