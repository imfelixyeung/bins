import z from "zod";

export const responseSchema = z.object({
  result: z.array(
    z.object({
      postcode: z.string(),
      longitude: z.number(),
      latitude: z.number(),
      distance: z.number(),
    })
  ),
});

export const getNearbyPostcodes = async (postcode: string) => {
  const url = new URL(`https://api.postcodes.io/postcodes/${postcode}/nearest`);

  url.searchParams.append("limit", "100");
  url.searchParams.append("radius", "2000");

  const res = await fetch(url.toString());

  if (!res.ok) {
    throw new Error("Failed to fetch nearby postcodes");
  }

  const json = await res.json().then(responseSchema.parseAsync);

  return json.result;
};

export type NearbyPostcode = z.infer<typeof responseSchema>["result"][number];
