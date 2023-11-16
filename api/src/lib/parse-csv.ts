import { logger } from "@/logger";
import csv from "csv-parser";
import { Readable } from "stream";

const jobLogger = logger.child({ job: "@/lib/parseCsv" });

class CSVError extends Error {
  constructor(message: string) {
    super(message);
    this.name = "CSVError";
  }
}

export const parseCsv = async <Headers extends readonly string[]>(options: {
  url: string;
  headers: Headers;
}) => {
  try {
    jobLogger.info("parseCsv starting");
    const { url, headers } = options;

    jobLogger.info("fetching csv");
    const response = await fetch(url);
    const csvText = await response.text();

    jobLogger.info("parsing csv");
    const csvStream = Readable.from(csvText);

    const results: {
      [key in (typeof headers)[number]]: string;
    }[] = await new Promise((resolve, reject) => {
      const results: {
        [key in (typeof headers)[number]]: string;
      }[] = [];
      csvStream
        .pipe(csv({ headers }))
        // eslint-disable-next-line @typescript-eslint/no-unsafe-argument
        .on("data", (data) => {
          if (results.length % 10000 === 0)
            console.debug(`Parsed ${results.length} rows`);
          return results.push(data);
        })
        .on("end", () => resolve(results))
        .on("error", (error) => reject(error));
    });

    jobLogger.info("parseCsv finished");

    return results;
  } catch (error) {
    if (error instanceof Error) {
      jobLogger.error(`parseCsv failed: ${error.message}`);
      return new CSVError(error.message);
    } else {
      jobLogger.error(`parseCsv failed: ${error}`);
      return new CSVError("Unknown error");
    }
  }
};
