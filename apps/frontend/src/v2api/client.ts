import { z } from "zod";

const makeAPISchema = <T>(dataSchema: z.ZodType<T>) => {
  return z.union([
    z.object({
      success: z.boolean(),
      data: dataSchema,
    }),
    z.object({
      error: z.boolean(),
      message: z.string(),
    }),
  ]);
};

const throwErrorMessage = <D>(
  response: { success: boolean; data: D } | { error: boolean; message: string },
): { success: true; data: D } => {
  if ("error" in response) {
    throw new Error(response.message);
  }
  const { success, data } = response;
  if (!success) {
    throw new Error("Something went wrong...");
  }
  return { success, data };
};

const premiseSchema = z.object({
  id: z.number(),
  addressRoom: z.string().nullable(),
  addressNumber: z.string().nullable(),
  addressStreet: z.string().nullable(),
  addressLocality: z.string().nullable(),
  addressCity: z.string().nullable(),
  addressPostcode: z.string().nullable(),
  updatedAt: z.string(),
});

export type Premise = z.infer<typeof premiseSchema>;

const jobSchema = z.object({
  bin: z.string(),
  date: z.string(),
});

export type Job = z.infer<typeof jobSchema>;

const BASE_URL = "/api";
export const PREMISES_SITEMAPS_URL = `${BASE_URL}/sitemaps/premises.xml`;

const getRandomPremisesSchema = makeAPISchema(premiseSchema);
export const getRandomPremises = async () => {
  return await fetch(`${BASE_URL}/random/premises`)
    .then((res) => res.json())
    .then(getRandomPremisesSchema.parseAsync)
    .then((data) => throwErrorMessage(data).data);
};
export type GetRandomPremisesSchema = Premise;

const searchPremisesSchema = makeAPISchema(z.array(premiseSchema));
export const searchPremises = async ({ postcode }: { postcode: string }) => {
  const query = new URLSearchParams();
  query.set("postcode", postcode);
  return await fetch(`${BASE_URL}/premises?${query.toString()}`)
    .then((res) => res.json())
    .then(searchPremisesSchema.parseAsync)
    .then((data) => throwErrorMessage(data).data);
};
export type SearchPremisesSchema = Premise[];

const premisesWithJobsSchema = premiseSchema.extend({
  jobs: jobSchema.array(),
});
const searchJobsSchema = makeAPISchema(premisesWithJobsSchema);
export const searchJobs = async ({ premisesId }: { premisesId: number }) => {
  const query = new URLSearchParams();
  query.set("premises", premisesId.toString());
  return await fetch(`${BASE_URL}/jobs?${query.toString()}`)
    .then((res) => res.json())
    .then(searchJobsSchema.parseAsync)
    .then((data) => throwErrorMessage(data).data);
};
export type SearchJobsSchema = z.infer<typeof premisesWithJobsSchema>;

const postcodeJobSchema = jobSchema.extend({
  postcode: z.string(),
});

const nearbyPostcodeSchema = z.object({
  postcode: z.string(),
  latitude: z.number(),
  longitude: z.number(),
  distance: z.number(),
  jobs: z.array(postcodeJobSchema),
});

export type NearbyPostcode = z.infer<typeof nearbyPostcodeSchema>;

const nearbySchema = makeAPISchema(z.array(nearbyPostcodeSchema).nullable());
export const getNearbyPostcodes = async ({
  postcode,
}: {
  postcode: string;
}) => {
  const query = new URLSearchParams();
  query.set("postcode", postcode);
  return await fetch(`${BASE_URL}/nearby?${query.toString()}`)
    .then((res) => res.json())
    .then(nearbySchema.parseAsync)
    .then((data) => throwErrorMessage(data).data);
};
export type NearbySchema = NearbyPostcode;

const syncRunSchema = z.object({
  state: z.enum(["running", "synced", "unchanged", "failed"]),
  startedAt: z.string(),
  finishedAt: z.string().nullable(),
  rows: z.number().nullable(),
  message: z.string().nullable(),
});

const statusDatasetSchema = z.object({
  key: z.string(),
  name: z.string(),
  url: z.string(),
  etag: z.string().nullable(),
  lastChecked: z.string().nullable(),
  lastSynced: z.string().nullable(),
  sync: syncRunSchema.nullable(),
});

export type StatusDataset = z.infer<typeof statusDatasetSchema>;

const datasetStatusSchema = z.object({
  datasets: z.array(statusDatasetSchema),
});
const statusSchema = makeAPISchema(datasetStatusSchema);
export const getStatus = async () => {
  return await fetch(`${BASE_URL}/status`)
    .then((res) => res.json())
    .then(statusSchema.parseAsync)
    .then((data) => throwErrorMessage(data).data);
};
export type StatusSchema = z.infer<typeof datasetStatusSchema>;
