import { ReturnedJobs } from "@/functions/search-jobs";
import React, { useMemo } from "react";
import DotCalendar from "./dot-calendar";
import { binToClassName } from "@/data/bin-colours";

const DotCalendarBins = ({ data }: { data: ReturnedJobs["jobs"] }) => {
  const transformedData = useMemo(() => {
    // group by date
    const byDate = data.reduce(
      (acc, item) => {
        const date = item.date;
        if (!acc[date]) {
          acc[date] = [];
        }
        acc[date].push(item.bin);
        return acc;
      },
      {} as Record<string, string[]>
    );

    return Object.entries(byDate).map(([date, bins]) => {
      return {
        date: new Date(date),
        classNames: bins.map((bin) => binToClassName[bin] ?? ""),
      };
    });
  }, [data]);

  return (
    <div>
      <DotCalendar data={transformedData} />
    </div>
  );
};

export default DotCalendarBins;
