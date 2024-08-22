"use client";

import { cn } from "@/lib/utils";
import { addDays, format, startOfWeek, subDays } from "date-fns";
import React, { useMemo } from "react";

const SpannedDot = ({ span }: { span: number }) => {
  if (span === 0) return null;

  return (
    <div
      className="flex items-center w-full h-5 rounded-full bg-gray-100"
      style={{
        gridColumn: `span ${span}`,
      }}
    ></div>
  );
};

const DotCalendar = ({
  data,
}: {
  data: {
    date: Date;
    classNames: string[];
  }[];
}) => {
  const now = new Date();
  const today = format(now, "yyyy-MM-dd");
  const start = startOfWeek(subDays(now, 7));
  const end = useMemo(() => {
    const max = data.reduce((max, item) => {
      if (item.date.getTime() > max.getTime()) {
        return item.date;
      }
      return max;
    }, now);

    return new Date(Math.max(max.getTime(), addDays(now, 7).getTime()));
  }, [data, now]);

  const classNamesByDate = useMemo(() => {
    const result: Record<string, string[]> = {};
    data.forEach(({ date, classNames }) => {
      const formattedDate = format(date, "yyyy-MM-dd");
      if (!result[formattedDate]) {
        result[formattedDate] = classNames;
      }
    });

    return result;
  }, [data]);

  const startToEnd = useMemo(() => {
    // an array of dates from now to maxDate
    const dates: { date: string; classNames: string[] }[] = [];
    let date = start;
    while (date.getTime() <= end.getTime()) {
      const formattedDate = format(date, "yyyy-MM-dd");
      dates.push({
        date: formattedDate,
        classNames: classNamesByDate[formattedDate] ?? [],
      });
      date = new Date(date.getTime() + 1000 * 60 * 60 * 24);
    }
    return dates;
  }, [end, start, classNamesByDate]);

  const firstDotOffset = (start.getDay() + 1) % 7;
  const lastDotOffset = 6 - ((end.getDay() + 1) % 7);

  return (
    <div>
      <div className="grid grid-cols-7 gap-1.5 max-w-fit">
        <SpannedDot span={firstDotOffset} />
        {startToEnd.map((date) => (
          <div
            key={date.date}
            className={cn(
              "h-5 w-5 rounded-full bg-gray-300",
              date.date === today && "ring-2 ring-offset-1 ring-blue-600"
            )}
          >
            <div className="flex h-full w-full justify-stretch rounded-full overflow-hidden">
              {date.classNames.map((className, index) => (
                <div key={index} className={cn(className, "grow")}></div>
              ))}
            </div>
          </div>
        ))}
        <SpannedDot span={lastDotOffset} />
      </div>
    </div>
  );
};

export default DotCalendar;
