"use client";

import React from "react";
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

const formSchema = z.object({
  postcode: z
    .string()
    .min(1, { message: "Postcode is required" })
    .transform((value) => value.trim().replaceAll(" ", "").toUpperCase()),
  premises: z.coerce.number().nullable(),
});

type FormData = z.infer<typeof formSchema>;

const PremisesSearchForm = () => {
  const router = useRouter();
  const [postcode, setPostcode] = useQueryState("postcode", {
    defaultValue: "",
  });
  const form = useForm<FormData>({
    resolver: zodResolver(formSchema),
    defaultValues: {
      postcode,
      premises: null,
    },
  });
  const { data: premises } = useSearchPremisesQuery({ postcode });

  const onSubmit = async (data: FormData) => {
    setPostcode(data.postcode);
  };

  const onSelectAddress = (premisesId: string) => {
    router.push(`/premises/${premisesId}`);
  };

  return (
    <Form {...form}>
      <form onSubmit={form.handleSubmit(onSubmit)} className="space-y-8">
        <FormField
          control={form.control}
          name="postcode"
          render={({ field }) => (
            <FormItem>
              <FormLabel>Enter your postcode</FormLabel>
              <FormControl>
                <Input placeholder="" {...field} />
              </FormControl>
              <FormDescription>For example, 'LS2 3AB'</FormDescription>
              <FormMessage />
            </FormItem>
          )}
        />
        <Button type="submit">Lookup</Button>

        {premises && (
          <FormField
            control={form.control}
            name="premises"
            render={({ field }) => (
              <FormItem>
                <FormLabel>Address</FormLabel>
                <Select
                  onValueChange={(value) => {
                    field.onChange(value);
                    onSelectAddress(value);
                  }}
                  defaultValue={field.value?.toString() ?? ""}
                >
                  <FormControl>
                    <SelectTrigger>
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
        )}
      </form>
    </Form>
  );
};

export default PremisesSearchForm;
