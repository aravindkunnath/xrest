
export type RequestParams = {
  key?: string;
  name?: string;
  value: string;
  isEnabled?: boolean;
  enabled?: boolean;
}

const EMPTY_LITERAL = "";
const COMMENT_LITERAL = "//";
const NEW_LINE_LITERAL = "\n";

export const serializeForBulkEdit = (params: any[]): string => {
  const buildString = (item: any): string => {
    const isEnabled = item.isEnabled !== undefined ? item.isEnabled : (item.enabled !== undefined ? item.enabled : true);
    const key = item.key !== undefined ? item.key : (item.name !== undefined ? item.name : "");
    const prefix = isEnabled ? EMPTY_LITERAL : `${COMMENT_LITERAL} `;
    return `${prefix}${key}: ${item.value ?? ''}`;
  };

  const seen = new Set<string>();
  const uniqueParams: any[] = [];
  for (const item of params) {
    const key = item.key !== undefined ? item.key : (item.name !== undefined ? item.name : "");
    if (!seen.has(key)) {
      seen.add(key);
      uniqueParams.push(item);
    }
  }
  return uniqueParams.map(buildString).join(NEW_LINE_LITERAL);
};

