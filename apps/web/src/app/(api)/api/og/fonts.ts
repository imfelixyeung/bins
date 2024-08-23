const boldFontURL = new URL(
  "https://unpkg.com/geist@1.3.1/dist/fonts/geist-sans/Geist-Bold.ttf"
);

const semiboldFontURL = new URL(
  "https://unpkg.com/geist@1.3.1/dist/fonts/geist-sans/Geist-SemiBold.ttf"
);

export const loadFont = async (url: URL) => {
  const fontData = await fetch(url).then((res) => (res as any).arrayBuffer());

  return fontData;
};

export const fonts = {
  geist: {
    bold: boldFontURL,
    semibold: semiboldFontURL,
  },
};
