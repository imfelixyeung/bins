import { exec as _exec } from "node:child_process";
import etag from "../etag";
import { urls } from "../data";
import { promisify } from "util";

const exec = promisify(_exec);

export const updatePremises = async () =>
  etag.run(urls.premises, async () => {
    try {
      await exec("./bin/import-csv premises");
    } catch (error) {
      throw error;
    }
    return true;
  });
