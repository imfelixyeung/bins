import { exec as _exec } from "node:child_process";
import etag from "../etag";
import { urls } from "../data";
import { promisify } from "util";

const exec = promisify(_exec);

export const updateJobs = async () =>
  etag.run(urls.jobs, async () => {
    try {
      await exec("./bin/import-csv jobs");
    } catch (error) {
      throw error;
    }
    return true;
  });
