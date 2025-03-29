import { binToClassName } from "@/data/bin-colours";
import { ReturnedJobs } from "@/functions/search-jobs";
import { cn } from "@/lib/utils";
import { capitalCase } from "change-case";
import { Trash2Icon } from "lucide-react";
import React from "react";

const PremisesWidget = ({ data }: { data: ReturnedJobs["jobs"] }) => {
  const twoUpcomingBins = data
    .toSorted(
      ({ date: a }, { date: b }) =>
        new Date(a).getTime() - new Date(b).getTime()
    )
    .slice(0, 2);
  const currentBin = twoUpcomingBins[0];
  const upcomingBin = twoUpcomingBins[1];

  if (!currentBin || !upcomingBin) {
    return null;
  }

  return (
    <div className="flex size-96 border rounded-3xl p-6 relative">
      <div className="grow flex items-start flex-col">
        <PremisesWidget.BinAndDate
          bin={currentBin.bin}
          date={currentBin.date}
        />
      </div>
      <div className="grow flex items-end flex-col justify-end">
        <PremisesWidget.BinAndDate
          bin={upcomingBin.bin}
          date={upcomingBin.date}
          end
        />
      </div>
      <div className="absolute inset-y-0 inset-x-0 flex justify-center items-center">
        <Trash2Icon className="size-48" />
      </div>
    </div>
  );
};

PremisesWidget.BinAndDate = ({
  bin,
  date,
  end,
}: {
  bin: string;
  date: string;
  end?: boolean;
}) => {
  const jsDate = new Date(date);

  const weekday = jsDate.toLocaleString("en-GB", {
    weekday: "short",
  });
  // pad with 0 if single digit
  const day = jsDate.getDate().toString().padStart(2, "0");

  return (
    <div
      className={cn(
        "flex gap-1",
        end ? "items-end flex-col-reverse" : "flex-col"
      )}
    >
      <div className="text-4xl font-bold">{capitalCase(bin)}</div>
      <div className="text-xl font-medium">
        {day}, {weekday}
      </div>
      <div className={cn(binToClassName[bin], "w-8 h-1")}></div>
    </div>
  );
};

export default PremisesWidget;
