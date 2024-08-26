"use client";

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

const postcodeFormSchema = z.object({
  postcode: z
    .string()
    .min(1, { message: "Postcode is required" })
    .transform((value) => value.trim().replaceAll(" ", "").toUpperCase()),
});

const premisesFormSchema = z.object({
  premises: z.coerce.number({
    invalid_type_error: "Please select an address from the list",
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
  const postcodeForm = useForm<PostcodeFormData>({
    resolver: zodResolver(postcodeFormSchema),
    defaultValues: {
      postcode,
    },
  });
  const premisesForm = useForm<PremisesFormData>({
    resolver: zodResolver(premisesFormSchema),
    defaultValues: {
      premises: undefined,
    },
  });
  const { data: premises } = useSearchPremisesQuery({ postcode });

  const onSubmitPostcode = async (data: PostcodeFormData) => {
    setPostcode(data.postcode);
  };

  const onSubmitPremises = async (data: PremisesFormData) => {
    router.push(`/premises/${data.premises}`);
  };

  useEffect(() => {
    if (premises) {
      addressSelectRef.current?.focus();
    }
  }, [premises]);

  return (
    <>
      <Form {...postcodeForm}>
        <form
          onSubmit={postcodeForm.handleSubmit(onSubmitPostcode)}
          className="space-y-8"
        >
          <FormField
            control={postcodeForm.control}
            name="postcode"
            render={({ field }) => (
              <FormItem>
                <FormLabel>Enter your postcode</FormLabel>
                <FormControl>
                  <Input placeholder="" {...field} className={cn("max-w-xs")} />
                </FormControl>
                <FormDescription>For example, 'LS2 3AB'</FormDescription>
                <FormMessage />
              </FormItem>
            )}
          />
          <Button
            type="submit"
            variant={postcode ? "outline" : "default"}
            onClick={
              premises ? postcodeForm.handleSubmit(onSubmitPostcode) : undefined
            }
          >
            Lookup
          </Button>
        </form>
      </Form>
      {premises && (
        <Form {...premisesForm}>
          <form
            onSubmit={premisesForm.handleSubmit(onSubmitPremises)}
            className="space-y-8 mt-16"
          >
            <>
              <FormField
                control={premisesForm.control}
                name="premises"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>Address</FormLabel>
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
                        {/* <SelectItem value="m@example.com">m@example.com</SelectItem>
                  <SelectItem value="m@google.com">m@google.com</SelectItem>
                  <SelectItem value="m@support.com">m@support.com</SelectItem> */}

                        {premises.data?.map((premises) => {
                          const {
                            addressRoom,
                            addressNumber,
                            addressStreet,
                            addressLocality,
                            addressCity,
                            addressPostcode,
                          } = premises;
                          const fullAddress = [
                            addressRoom,
                            addressNumber,
                            addressStreet,
                            addressLocality,
                            addressCity,
                            addressPostcode,
                          ]
                            .filter(Boolean)
                            .join("\n");
                          return (
                            <SelectItem value={premises.id.toString()}>
                              {fullAddress}
                            </SelectItem>
                          );
                        })}
                      </SelectContent>
                    </Select>
                    <FormMessage />
                  </FormItem>
                )}
              />
              <Button type="submit">Confirm Address</Button>
            </>
          </form>
        </Form>
      )}
    </>
  );
};

export default PremisesSearchForm;
