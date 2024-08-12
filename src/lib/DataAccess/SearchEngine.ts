import { invoke } from "@tauri-apps/api/core";

type IndexTarget = {
  id: string;
  doc_type: "card" | "thread";
  text: string;
};

type SearchOption = {
  levenshteinDistance?: number;
  limit?: number;
};

type SearchResult = {
  id: string;
  doc_type: "card" | "thread";
};

const index = async (input: IndexTarget[]): Promise<void> => {
  await invoke("index", {
    input: input,
  });
};

const search = async (
  query: string,
  option?: SearchOption,
): Promise<SearchResult[]> => {
  return await invoke("search", {
    query,
    levenshteinDistance: option?.levenshteinDistance || 2,
    limit: option?.limit || 100,
  });
};

export const SearchEngine = {
  index,
  search,
} as const;
