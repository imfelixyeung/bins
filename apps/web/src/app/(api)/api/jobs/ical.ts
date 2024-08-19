import { getFullAddress, getSummaryAddress } from "@/functions/format-address";
import { ReturnedJobs } from "@/functions/search-jobs";
import { capitalCase } from "change-case";
import ical from "ical-generator";

const createURL = (id: string, date: string, bin: string) =>
  `https://bins.felixyeung.com/premises/${id}?date=${date}&bin=${bin}`;

export const createIcal = ({ data }: { data: ReturnedJobs }) => {
  const fullAddress = getFullAddress(data);
  const summaryAddress = getSummaryAddress(data);

  const calendar = ical({
    name: `Bin Days for ${summaryAddress}`,
    description: `Bin collection dates for your household\n\n${fullAddress}\n\nID: ${data.id}`,
    timezone: "Europe/London",
  });

  for (const job of data.jobs) {
    const { date, bin } = job;
    const id = `${data.id}_${date}_${bin}`;

    calendar.createEvent({
      id,
      start: date,
      end: date,
      allDay: true,
      url: createURL(data.id.toString(), date, bin),
      summary: `${capitalCase(bin)} Bin Collection`,
      timezone: "Europe/London",
      location: fullAddress,
    });
  }

  return calendar.toString();
};
