import { execSync } from "node:child_process";

export const updatePremises = async () => {
  try {
    execSync("./bin/import-csv premises");
  } catch (error) {
    throw error;
  }
};
