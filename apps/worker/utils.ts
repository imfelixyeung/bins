export const nullIfEmpty = (value: string) => {
  if (value === "" || value === "\u0000") return null;
  return value;
};
