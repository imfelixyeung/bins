/**
 * tests if inserting millions of rows to a serverless sqlite database is viable
 */

import { urls } from "./data";
import fs from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { db } from "@repo/database/src";
import readline from "readline";
import { premisesTable } from "@repo/database/src/schema";

const createTempDir = async () => {
  if (process.env.NODE_ENV === "development") {
    console.info("Development mode detected, using current directory");
    const path = join(process.cwd(), "temp");
    await fs.mkdir(path, {
      recursive: true,
    });
    return path;
  }
  return await fs.mkdtemp(join(tmpdir(), "dm-data"));
};

export const downloadPremises = async () => {
  const response = await fetch(urls.premises);
  if (!response.body) throw new Error("No response body");

  const reader = response.body;
  const decoder = new TextDecoder();

  const dir = await createTempDir();
  const filePath = join(dir, "dm_premises.csv");
  const file = await fs.open(filePath, "w");

  for await (const chunk of reader) {
    const line = decoder.decode(chunk).replaceAll("\u0000", "");
    await file.write(line);
  }

  await file.close();

  console.log(`Premises file written to ${filePath}`);

  return filePath;
};

export const downloadJobs = async () => {};

export const updateLatest = async () => {
  const [jobsPath, premisesPath] = await Promise.all([
    downloadJobs(),
    downloadPremises(),
  ]);
  console.log({ jobsPath, premisesPath });

  await insertPremises(premisesPath);
};

export const insertPremises = async (path: string) => {
  const file = await fs.open(path, "r");
  const stream = file.createReadStream();
  const reader = readline.createInterface({
    input: stream,
    crlfDelay: Infinity,
  });

  const chunks: (typeof premisesTable.$inferInsert)[] = [];

  for await (const line of reader) {
    const [
      id,
      addressRoom,
      addressNumber,
      addressStreet,
      addressLocality,
      addressCity,
      addressPostcode,
    ] = line.split(",");

    if (Math.random() > 0.0001) {
      console.log({
        id,
        addressRoom,
        addressNumber,
        addressStreet,
        addressLocality,
        addressCity,
        addressPostcode,
      });
    }

    chunks.push({
      id: Number(id),
      addressRoom,
      addressNumber,
      addressStreet,
      addressLocality,
      addressCity,
      addressPostcode,
      searchPostcode: addressPostcode?.replaceAll(" ", "").toUpperCase(),
    });

    if (chunks.length >= 100) {
      await db.insert(premisesTable).values(chunks);
      chunks.splice(0, chunks.length);
      break;
    }
  }

  file.close();
};

updateLatest();
