import { cva } from "class-variance-authority";

const supportedBins = ["BLACK", "GREEN", "BROWN"] as const;
type SupportedBin = (typeof supportedBins)[number];

export const isSupportedBin = (bin: string): bin is SupportedBin => {
  return supportedBins.includes(bin as SupportedBin);
};

export const binStyles = cva("", {
  variants: {
    bin: {
      BLACK: "bg-gray-900 text-gray-50",
      GREEN: "bg-green-700 text-gray-50",
      BROWN: "bg-amber-900 text-gray-50",
    },
    styled: {
      true: "py-5 rounded-t-lg",
      false: "",
    },
  },
  defaultVariants: { styled: true, bin: undefined },
});
