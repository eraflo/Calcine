/** The same request as a curl command, for reproducing it outside Calcine. */
export function toCurl(url: string, body: unknown): string {
  const json = JSON.stringify(body, null, 2).replaceAll("'", "'\\''");
  return [
    `curl ${url} \\`,
    '  -H "Authorization: Bearer $CALCINE_API_KEY" \\',
    '  -H "Content-Type: application/json" \\',
    `  -d '${json}'`,
  ].join("\n");
}
