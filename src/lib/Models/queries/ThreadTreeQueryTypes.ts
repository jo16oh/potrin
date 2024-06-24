type Card = {
  id: string;
  content: string;
  fractional_index: string;
};

type Breadcrumb = {
  id: string;
  title: string;
};

export type ThreadTreeQueryRawResult = {
  json: string;
}[];

export type ThreadTreeQueryResult = {
  id: string;
  title: string;
  fractional_index: string;
  cards: Card[];
  child_threads?: ThreadTreeQueryResult[];
  breadcrumbs?: Breadcrumb[];
};
