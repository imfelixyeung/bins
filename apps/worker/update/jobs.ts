import { execSync } from "node:child_process";
import etag from "../etag";
import { urls } from "../data";

export const updateJobs = async () =>
  etag.run(urls.jobs, async () => {
    try {
      execSync("./bin/import-csv jobs");
    } catch (error) {
      throw error;
    }
    return;
  });
