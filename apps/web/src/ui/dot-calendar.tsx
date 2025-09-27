"use client";

import { cn } from "@/lib/utils";
import { addDays, endOfWeek, format, startOfWeek, subDays } from "date-fns";
import React, { useMemo } from "react";

const WeekDays = () => {
  const days = ["Mo", "Tu", "We", "Th", "Fr", "Sa", "Su"];

  return (
    <div className="grid grid-cols-7 gap-1.5 max-w-fit bg-gray-100 dark:bg-gray-800 rounded-sm">
      {days.map((day, index) => (
        <div key={index} className="h-5 w-5">
          <div className="flex h-full w-full justify-center items-center">
            <div className="text-xs text-muted-foreground">{day}</div>
          </div>
        </div>
      ))}
    </div>
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
  const start = startOfWeek(subDays(now, 7), { weekStartsOn: 1 });
  const end = useMemo(() => {
    const max = data.reduce((max, item) => {
      if (item.date.getTime() > max.getTime()) {
        return item.date;
      }
      return max;
    }, now);

    return new Date(
      Math.min(
        Math.max(max.getTime(), addDays(now, 7).getTime()),
        endOfWeek(addDays(now, 28), { weekStartsOn: 1 }).getTime()
      )
    );
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
      date = addDays(date, 1);
    }

    console.log(dates);
    return dates;
  }, [end, start, classNamesByDate]);

  return (
    <div>
      <WeekDays />
      <div className="grid grid-cols-7 gap-1.5 max-w-fit mt-1.5">
        {startToEnd.map((date) => (
          <div
            key={date.date}
            className={cn(
              "h-5 w-5 rounded-full bg-gray-300 dark:bg-gray-600",
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
      </div>
    </div>
  );
};

export default DotCalendar;
