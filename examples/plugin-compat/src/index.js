import { answer } from "./answer";
import { secret } from "./secret";
import "./index.css";
// Importing the below dependency to force to create the 3rdpartylicenses.txt file (license-webpack-plugin's output)
import { plugin } from "copy-webpack-plugin";
console.log({ answer, secret });
