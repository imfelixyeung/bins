import { ReturnedJobs } from "@/functions/search-jobs";

export const createCSV = ({ data }: { data: ReturnedJobs }) => {
  const header = ["Date", "Bin"];
  const rows = data.jobs.map(({ date, bin }) => [date, bin]);

  return [header, ...rows].map((row) => row.join(",")).join("\n");
};
