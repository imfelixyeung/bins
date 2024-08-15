import { ReturnedJobs } from "@/functions/search-jobs";
import React from "react";
import Address from "./address";
import { isAfter, format } from "date-fns";
import { capitalCase } from "change-case";

const PremisesJobList = ({ data }: { data: ReturnedJobs }) => {
  const now = new Date();
  const jobsByBin = Object.entries(
    data.jobs
      .filter((job) => isAfter(new Date(job.date), now))
      .sort((a, b) => new Date(a.date).getTime() - new Date(b.date).getTime())
      .reduce(
        (acc, job) => {
          const { bin } = job;
          if (!acc[bin]) acc[bin] = [];
          acc[bin].push(job.date);
          return acc;
        },
        {} as Record<string, ReturnedJobs["jobs"][number]["date"][]>
      )
  ).map(([bin, dates]) => ({ bin, dates }));

  return (
    <div>
      <h2 className="text-2xl font-semibold">Address</h2>
      <Address data={data} />
      <h2 className="text-2xl font-semibold mt-3">By Bin</h2>
      {jobsByBin.map(({ bin, dates }) => (
        <div key={bin} className="mt-3">
          <h3 className="text-xl font-semibold">{capitalCase(bin)}</h3>
          <ul className="list-disc list-inside">
            {dates.map((date) => {
              // format date to 29 March 2023
              const formattedDate = format(new Date(date), "dd MMMM yyyy");
              return (
                <li key={date}>
                  <time dateTime={date}>{formattedDate}</time>
                </li>
              );
            })}
          </ul>
        </div>
      ))}
    </div>
  );
};

export default PremisesJobList;
