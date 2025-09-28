import type { MetadataRoute } from "next";

const manifest = (): MetadataRoute.Manifest => {
  return {
    id: "https://bins.felixyeung.com/",
    name: "Bins",
    short_name: "Bins",
    description: "Leeds bin days",
    lang: "en-GB",
    start_url: "/",
    scope: "/",
    theme_color: "#000000",
    background_color: "#000000",
    dir: "auto",
    orientation: "any",
    display: "minimal-ui",
    icons: [
      {
        purpose: "maskable",
        sizes: "512x512",
        src: "/assets/icon512_maskable.png",
        type: "image/png",
      },
      {
        purpose: "any",
        sizes: "512x512",
        src: "/assets/icon512_rounded.png",
        type: "image/png",
      },
    ],
  };
};

export default manifest;
