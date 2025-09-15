import { execSync } from "node:child_process";

export const updateJobs = async () => {
  try {
    execSync("./bin/import-csv jobs");
  } catch (error) {
    throw error;
  }
};
