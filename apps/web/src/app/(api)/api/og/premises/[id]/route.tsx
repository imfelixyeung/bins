import { getSummaryAddress } from "@/functions/format-address";
import { searchJobs } from "@/functions/search-jobs";
import { capitalCase } from "change-case";
import { ImageResponse } from "next/og";
import { NextRequest } from "next/server";
import { z } from "zod";
// App router includes @vercel/og.
// No need to install it.

const premisesIdSchema = z.coerce.number();

const boldFontURL = new URL(
  "https://unpkg.com/geist@1.3.1/dist/fonts/geist-sans/Geist-Bold.ttf"
);

const semiboldFontURL = new URL(
  "https://unpkg.com/geist@1.3.1/dist/fonts/geist-sans/Geist-SemiBold.ttf"
);

const loadFont = async (url: URL) => {
  const fontData = await fetch(url).then((res) => (res as any).arrayBuffer());

  return fontData;
};

export async function GET(
  request: NextRequest,
  { params: { id: _id } }: { params: { id: string } }
) {
  const parsedId = premisesIdSchema.safeParse(_id);

  if (!parsedId.success) return new Response("Not found", { status: 404 });

  const id = parsedId.data;

  const premises = await searchJobs({ premisesId: id });

  if (!premises) return new Response("Not found", { status: 404 });

  const address = capitalCase(getSummaryAddress(premises));
  const postcode = premises.addressPostcode;

  const boldFont = await loadFont(boldFontURL);
  const semiboldFont = await loadFont(semiboldFontURL);

  return new ImageResponse(
    (
      <div
        tw="flex flex-col w-full h-full items-center justify-between bg-white"
        style={{
          fontFamily: "Geist",
        }}
      >
        <div tw="flex flex-col items-center justify-center"></div>
        <div tw="flex flex-col items-center justify-center">
          <div tw="flex flex w-full px-16 items-center justify-center">
            <svg
              height={80}
              xmlns="http://www.w3.org/2000/svg"
              fill="none"
              viewBox="0 0 24 24"
              stroke-width="1.5"
              stroke="currentColor"
            >
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                d="m14.74 9-.346 9m-4.788 0L9.26 9m9.968-3.21c.342.052.682.107 1.022.166m-1.022-.165L18.16 19.673a2.25 2.25 0 0 1-2.244 2.077H8.084a2.25 2.25 0 0 1-2.244-2.077L4.772 5.79m14.456 0a48.108 48.108 0 0 0-3.478-.397m-12 .562c.34-.059.68-.114 1.022-.165m0 0a48.11 48.11 0 0 1 3.478-.397m7.5 0v-.916c0-1.18-.91-2.164-2.09-2.201a51.964 51.964 0 0 0-3.32 0c-1.18.037-2.09 1.022-2.09 2.201v.916m7.5 0a48.667 48.667 0 0 0-7.5 0"
              />
            </svg>

            <h1 tw="text-8xl font-bold pl-6">
              <span>Bin Dates</span>
            </h1>
          </div>
          <div tw="flex flex-col w-full px-16">
            <h2 tw="flex flex-col items-center justify-center font-semibold">
              <span tw="text-5xl">{address}</span>
              <span tw="text-3xl">{postcode}</span>
            </h2>
          </div>
        </div>
        <div tw="flex flex-col items-center justify-center">
          {/* bins.felixyeung.com */}
        </div>
      </div>
    ),
    {
      width: 1200,
      height: 630,
      fonts: [
        {
          name: "Geist",
          data: boldFont,
          weight: 700,
          style: "normal",
        },
        {
          name: "Geist",
          data: semiboldFont,
          weight: 600,
          style: "normal",
        },
      ],
    }
  );
}
