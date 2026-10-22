export default function (content) {
  if (!Buffer.isBuffer(content)) throw new Error("Expected a Buffer");
  return `module.exports = ${JSON.stringify(content.toString("hex"))};`;
};
export const raw = true;
