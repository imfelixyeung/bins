import { ReturnedJobs } from "@/functions/search-jobs";
import React from "react";
import Address from "./address";
import { isAfter, format } from "date-fns";
import { capitalCase } from "change-case";
import DotCalendarBins from "./dot-calendar-bins";
import { Trash2Icon } from "lucide-react";

const BinDates = ({ bin, dates }: { bin: string; dates: string[] }) => {
  return (
    <div className="grow basis-0 bg-gray-100 rounded-lg p-4">
      <h3 className="text-xl font-semibold flex items-center gap-2">
        {capitalCase(bin)} <Trash2Icon />
      </h3>
      <ul className="list-disc list-inside mt-3">
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
  );
};

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
      <div className="flex justify-between gap-8 flex-wrap">
        <section>
          <h2 className="text-2xl font-semibold">Address</h2>
          <div className="mt-3">
            <Address data={data} />
          </div>
        </section>
        <section>
          <h2 className="text-2xl font-semibold">At a glance</h2>
          <div className="mt-3">
            <DotCalendarBins data={data.jobs} />
          </div>
        </section>
      </div>
      <section>
        <h2 className="text-2xl font-semibold mt-3">By Bin</h2>
        <div className="mt-3">
          <div className="flex justify-stretch flex-wrap gap-6">
            {jobsByBin.map(({ bin, dates }) => (
              <BinDates key={bin} bin={bin} dates={dates} />
            ))}
          </div>
        </div>
      </section>
    </div>
  );
};

export default PremisesJobList;
