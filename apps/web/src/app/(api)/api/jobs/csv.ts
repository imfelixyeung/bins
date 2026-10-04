import { SearchJobsSchema } from "@/v2api/client";

export const createCSV = ({ data }: { data: SearchJobsSchema }) => {
  const header = ["Date", "Bin"];
  const rows = data.jobs.map(({ date, bin }) => [date, bin]);

  return [header, ...rows].map((row) => row.join(",")).join("\n");
};
