import { ReturnedJobs } from "@/functions/search-jobs";
import React, { useMemo } from "react";
import DotCalendar from "./dot-calendar";

const binToClassName: Record<string, string> = {
  BLACK: "bg-gray-900/90",
  BROWN: "bg-amber-900/90",
  GREEN: "bg-green-600/90",
};

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
