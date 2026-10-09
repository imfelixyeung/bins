"use client";

/**
 * new form styles inspired by:
 * https://dub.co/
 */

import {
  Button,
  Card,
  Description,
  ErrorMessage,
  FieldError,
  Form,
  Input,
  Label,
  ListBox,
  ListBoxItem,
  ListBoxItemIndicator,
  Select,
  SelectIndicator,
  SelectPopover,
  SelectTrigger,
  SelectValue,
  TextField,
} from "@heroui/react";
import { zodResolver } from "@hookform/resolvers/zod";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import {
  CheckIcon,
  DicesIcon,
  HouseIcon,
  SearchIcon,
  Trash2Icon,
  TrashIcon,
} from "lucide-react";
import { PrefetchKind } from "next/dist/client/components/router-reducer/router-reducer-types";
import { useRouter } from "next/navigation";
import { useQueryState } from "nuqs";
import { useEffect, useRef } from "react";
import { Controller, useForm } from "react-hook-form";
import { z } from "zod";
import { getPresentableFullAddress } from "@/functions/format-address";
import { getRandomPremises, searchPremises } from "@/v2api/client";
import ClientOnly from "./client-only";
import RecentPremises from "./recent-premises";

const postcodeFormSchema = z.object({
  postcode: z
    .string()
    .min(1, { message: "Postcode is required" })
    .transform((value) => value.trim().replaceAll(" ", "").toUpperCase()),
});

const premisesFormSchema = z.object({
  premises: z.coerce.number<string>({
    message: "Please select an address from the list",
  }),
});

type PostcodeFormData = z.infer<typeof postcodeFormSchema>;
type PremisesFormData = z.infer<typeof premisesFormSchema>;

const randomPremiseQueryOption = {
  queryKey: ["getRandomPremise"],
  queryFn: getRandomPremises,
} as const;

const randomPremiseQueryOptionInitialNull = {
  ...randomPremiseQueryOption,
  initialData: null,
};

const PremisesSearchForm = () => {
  const queryClient = useQueryClient();
  const randomPremisesQuery = useQuery(randomPremiseQueryOptionInitialNull);

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
      premises: "",
    },
  });
  const { data: premises, isLoading: premisesIsLoading } = useQuery({
    queryKey: ["premise", postcode],
    queryFn: () => searchPremises({ postcode }),
    enabled: postcode.trim().length > 0,
  });

  const selectedPremisesId = premisesForm.watch("premises", "");

  useEffect(() => {
    if (!selectedPremisesId) return;

    router.prefetch(`/premises?id=${selectedPremisesId}`, {
      kind: PrefetchKind.FULL,
    });
  }, [router, selectedPremisesId]);

  const onSubmitPostcode = async (data: PostcodeFormData) => {
    setPostcode(data.postcode);
  };

  const onSubmitPremises = async (data: PremisesFormData) => {
    router.push(`/premises?id=${data.premises}`);
  };

  const onSupriseMe = async () => {
    const premises = await queryClient.fetchQuery(randomPremiseQueryOption);

    if (!premises?.addressPostcode) return;

    setPostcode(premises.addressPostcode);
    postcodeForm.setValue("postcode", premises.addressPostcode);
    premisesForm.setValue("premises", premises.id.toString());
  };

  useEffect(() => {
    if (premises) {
      addressSelectRef.current?.focus();
    }
  }, [premises]);

  return (
    <Card className="max-w-lg relative px-4 pb-6 pt-8 w-full group/search rounded-4xl">
      <div>
        <div className="absolute -top-3 inset-x-0 flex items-center justify-center">
          <div className="flex text-sm text-foreground/80 items-center gap-1 border px-2 rounded-full h-6 bg-background">
            <HouseIcon size={12} />
            <span>Address Loopup</span>
          </div>
        </div>
        <div className="absolute -top-5 inset-x-0 -z-10 text-muted scale-90 translate-y-5 group-hover/search:translate-y-0 group-focus-within/search:translate-y-0 group-hover/search:scale-100 group-focus-within/search:scale-100 transition-transform duration-500 motion-reduce:scale-100 motion-reduce:translate-y-0">
          <div className="absolute left-6 -rotate-12">
            <TrashIcon className="stroke-[1.5]" />
          </div>
          <div className="absolute right-12 rotate-6">
            <Trash2Icon className="stroke-[1.5]" />
          </div>
        </div>
        <Form
          onSubmit={postcodeForm.handleSubmit(onSubmitPostcode)}
          className="flex gap-3"
        >
          <Controller
            control={postcodeForm.control}
            name="postcode"
            render={({
              field: { name, value, onChange, onBlur, ref },
              fieldState: { invalid, error },
            }) => (
              <TextField
                className="max-w-none w-full"
                name={name}
                value={value}
                onChange={onChange}
                onBlur={onBlur}
                ref={ref}
                validationBehavior="aria"
                isInvalid={invalid}
              >
                <Label className="sr-only">Enter your postcode</Label>
                <Input
                  placeholder="Postcode eg. LS2 3AB"
                  className="max-w-none w-full"
                  variant="secondary"
                />
                <Description className="sr-only">
                  {"For example, 'LS2 3AB'"}
                </Description>
                <FieldError>{error?.message}</FieldError>
              </TextField>
            )}
          />
          <Button
            type="submit"
            variant={postcode ? "outline" : "primary"}
            className="shrink-0"
            onClick={
              premises ? postcodeForm.handleSubmit(onSubmitPostcode) : undefined
            }
            isDisabled={premisesIsLoading}
            isIconOnly
          >
            <SearchIcon size={16} />
            <div className="sr-only">Lookup</div>
          </Button>
        </Form>
        {premises && premises && premises.length === 0 && (
          <div className="mt-8">
            <p className="text-lg">
              Oops, we {"couldn't"} find any addresses matching your postcode of{" "}
              {postcode}
            </p>
          </div>
        )}
        {premises && premises && premises.length > 0 && (
          <Form
            onSubmit={premisesForm.handleSubmit(onSubmitPremises)}
            className="mt-4 flex gap-3"
          >
            <Controller
              control={premisesForm.control}
              name="premises"
              render={({
                field: { name, value, onChange, onBlur, ref },
                fieldState: { invalid, error },
              }) => (
                <Select
                  placeholder="Select an address from the list"
                  name={name}
                  value={value}
                  onChange={onChange}
                  onBlur={onBlur}
                  ref={ref}
                  validationBehavior="aria"
                  isInvalid={invalid}
                  className="grow"
                  variant="secondary"
                >
                  <Label className="sr-only">Address</Label>
                  <SelectTrigger ref={addressSelectRef}>
                    <SelectValue className="line-clamp-1" />
                    <SelectIndicator />
                  </SelectTrigger>
                  <SelectPopover>
                    <ListBox>
                      {premises?.map((premises) => {
                        const address = getPresentableFullAddress(premises);

                        return (
                          <ListBoxItem
                            key={premises.id}
                            textValue={premises.id.toString()}
                            id={premises.id.toString()}
                          >
                            {address}
                            <ListBoxItemIndicator />
                          </ListBoxItem>
                        );
                      })}
                    </ListBox>
                  </SelectPopover>
                  <Description className="sr-only">
                    {"For example, 'LS2 3AB'"}
                  </Description>
                  <ErrorMessage>{error?.message}</ErrorMessage>
                </Select>
              )}
            />
            <Button type="submit" isIconOnly className="shrink-0">
              <CheckIcon size={16} />
              <span className="sr-only">Confirm Address</span>
            </Button>
          </Form>
        )}
        <ClientOnly>
          <div className="flex justify-center mt-6">
            <Button
              className="gap-3"
              variant="ghost"
              size="sm"
              onClick={onSupriseMe}
              isDisabled={
                randomPremisesQuery.isFetching || !randomPremisesQuery.isStale
              }
            >
              <DicesIcon size={20} />
              Suprise Me
            </Button>
          </div>
        </ClientOnly>
        <ClientOnly>
          <RecentPremises />
        </ClientOnly>
      </div>
    </Card>
  );
};

export default PremisesSearchForm;
