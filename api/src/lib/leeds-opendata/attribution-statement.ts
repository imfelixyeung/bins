export const createAttributionStatement = (options: {
  title: string;
  yearOfPublication: string;
  documentUrl: string;
}) => {
  const { title, yearOfPublication, documentUrl } = options;

  return `${title}, (c) Leeds City Council, ${yearOfPublication}, ${documentUrl}. This information is licensed under the terms of the Open Government Licence.`;
};
