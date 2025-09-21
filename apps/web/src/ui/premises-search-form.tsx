"use client";

/**
 * new form styles inspired by:
 * https://dub.co/
 */

import React, { useEffect, useRef } from "react";
import { z } from "zod";
import { useForm } from "react-hook-form";
import { zodResolver } from "@hookform/resolvers/zod";
import { Button } from "@/components/ui/button";
import {
  Form,
  FormControl,
  FormDescription,
  FormField,
  FormItem,
  FormLabel,
  FormMessage,
} from "@/components/ui/form";
import { Input } from "@/components/ui/input";
import { useSearchPremisesQuery } from "@/hooks/use-search-premises";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { useRouter } from "next/navigation";
import { useQueryState } from "nuqs";
import { cn } from "@/lib/utils";
import { Card } from "@/components/ui/card";
import {
  CheckIcon,
  DicesIcon,
  HouseIcon,
  SearchIcon,
  Trash2Icon,
  TrashIcon,
} from "lucide-react";
import { getPresentableFullAddress } from "@/functions/format-address";
import ClientOnly from "./client-only";
import RecentPremises from "./recent-premises";
import { getRandomPremises } from "@/actions/get-random-premises";
import { PrefetchKind } from "next/dist/client/components/router-reducer/router-reducer-types";

const postcodeFormSchema = z.object({
  postcode: z
    .string()
    .min(1, { message: "Postcode is required" })
    .transform((value) => value.trim().replaceAll(" ", "").toUpperCase()),
});

const premisesFormSchema = z.object({
  premises: z.coerce.number({
    error: "Please select an address from the list",
  }),
});

type PostcodeFormData = z.infer<typeof postcodeFormSchema>;
type PremisesFormData = z.infer<typeof premisesFormSchema>;

const PremisesSearchForm = () => {
  const router = useRouter();
  const [postcode, setPostcode] = useQueryState("postcode", {
    defaultValue: "",
  });
  const addressSelectRef = useRef<HTMLButtonElement>(null);
  const postcodeForm = useForm({
    resolver: zodResolver(postcodeFormSchema),
    defaultValues: {
      postcode,
    },
  });
  const premisesForm = useForm({
    resolver: zodResolver(premisesFormSchema),
    defaultValues: {
      premises: undefined,
    },
  });
  const { data: premises } = useSearchPremisesQuery({ postcode });

  const selectedPremisesId = premisesForm.watch("premises");

  useEffect(() => {
    if (!selectedPremisesId) return;

    router.prefetch(`/premises/${selectedPremisesId}`, {
      kind: PrefetchKind.FULL,
    });
  }, [selectedPremisesId]);

  const onSubmitPostcode = async (data: PostcodeFormData) => {
    setPostcode(data.postcode);
  };

  const onSubmitPremises = async (data: PremisesFormData) => {
    router.push(`/premises/${data.premises}`);
  };

  const onSupriseMe = async () => {
    const result = await getRandomPremises();
    if (!result) return;

    const premises = result.data;
    if (!premises || !premises.addressPostcode) return;

    setPostcode(premises.addressPostcode);
    postcodeForm.setValue("postcode", premises.addressPostcode);
    premisesForm.setValue("premises", premises.id);
  };

  useEffect(() => {
    if (premises) {
      addressSelectRef.current?.focus();
    }
  }, [premises]);

  return (
    <Card className="max-w-lg relative px-4 pb-6 pt-8 w-full group/search">
      <div className="absolute -top-3 inset-x-0 flex items-center justify-center">
        <div className="flex text-sm text-foreground/80 items-center gap-1 border px-2 rounded-full h-6 bg-background">
          <HouseIcon size={12} />
          <span>Address Loopup</span>
        </div>
      </div>
      <div className="absolute -top-5 inset-x-0 -z-10 text-muted-foreground scale-90 translate-y-5 group-hover/search:translate-y-0 group-focus-within/search:translate-y-0 group-hover/search:scale-100 group-focus-within/search:scale-100 transition-transform duration-500 motion-reduce:scale-100 motion-reduce:translate-y-0">
        <div className="absolute left-6 -rotate-12">
          <TrashIcon className="stroke-[1.5]" />
        </div>
        <div className="absolute right-12 rotate-6">
          <Trash2Icon className="stroke-[1.5]" />
        </div>
      </div>
      <Form {...postcodeForm}>
        <form
          onSubmit={postcodeForm.handleSubmit(onSubmitPostcode)}
          className="flex gap-3"
        >
          <FormField
            control={postcodeForm.control}
            name="postcode"
            render={({ field }) => (
              <FormItem className="space-y-0 grow">
                <FormLabel className="sr-only">Enter your postcode</FormLabel>
                <FormControl>
                  <Input
                    placeholder="Postcode eg. LS2 3AB"
                    {...field}
                    className={cn("")}
                  />
                </FormControl>
                <FormDescription className="sr-only">
                  For example, 'LS2 3AB'
                </FormDescription>
                <FormMessage />
              </FormItem>
            )}
          />
          <Button
            type="submit"
            variant={postcode ? "outline" : "default"}
            size="icon"
            className="size-10 shrink-0"
            onClick={
              premises ? postcodeForm.handleSubmit(onSubmitPostcode) : undefined
            }
          >
            <SearchIcon size={16} />
            <div className="sr-only">Lookup</div>
          </Button>
        </form>
      </Form>
      {premises && premises.data && premises.data.length === 0 && (
        <div className="mt-8">
          <p className="text-lg">
            Oops, we couldn't find any addresses matching your postcode of{" "}
            {postcode}
          </p>
        </div>
      )}
      {premises && premises.data && premises.data.length > 0 && (
        <Form {...premisesForm}>
          <form
            onSubmit={premisesForm.handleSubmit(onSubmitPremises)}
            className="mt-4 flex gap-3"
          >
            <>
              <FormField
                control={premisesForm.control}
                name="premises"
                render={({ field }) => (
                  <FormItem className="space-y-0 grow">
                    <FormLabel className="sr-only">Address</FormLabel>
                    <Select
                      onValueChange={(value) => {
                        field.onChange(value);
                      }}
                      defaultValue={field.value?.toString() ?? ""}
                    >
                      <FormControl>
                        <SelectTrigger ref={addressSelectRef}>
                          <SelectValue placeholder="Select an address from the list" />
                        </SelectTrigger>
                      </FormControl>
                      <SelectContent>
                        {premises.data?.map((premises) => {
                          const address = getPresentableFullAddress(premises);

                          return (
                            <SelectItem
                              key={premises.id}
                              value={premises.id.toString()}
                            >
                              {address}
                            </SelectItem>
                          );
                        })}
                      </SelectContent>
                    </Select>
                    <FormMessage />
                  </FormItem>
                )}
              />
              <Button type="submit" size="icon" className="size-10 shrink-0">
                <CheckIcon size={16} />
                <span className="sr-only">Confirm Address</span>
              </Button>
            </>
          </form>
        </Form>
      )}
      <ClientOnly>
        <div className="flex justify-center mt-6">
          <Button
            className="gap-3"
            variant="ghost"
            size="sm"
            onClick={onSupriseMe}
          >
            <DicesIcon size={20} />
            Suprise Me
          </Button>
        </div>
      </ClientOnly>
      <ClientOnly>
        <RecentPremises />
      </ClientOnly>
    </Card>
  );
};

export default PremisesSearchForm;
