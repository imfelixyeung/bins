import { execSync } from "node:child_process";
import etag from "../etag";
import { urls } from "../data";

export const updatePremises = async () =>
  etag.run(urls.premises, async () => {
    try {
      execSync("./bin/import-csv premises");
    } catch (error) {
      throw error;
    }
    return true;
  });
