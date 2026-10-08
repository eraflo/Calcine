export type SnippetLanguage = "curl" | "python" | "javascript";

export const SNIPPET_LANGUAGES: { id: SnippetLanguage; label: string }[] = [
  { id: "curl", label: "curl" },
  { id: "python", label: "Python" },
  { id: "javascript", label: "JavaScript" },
];

/** Environment variable the snippets read the API key from. */
export const KEY_VARIABLE = "CALCINE_API_KEY";

/** Copy-paste examples for calling the local API from another app. */
export function snippet(language: SnippetLanguage, baseUrl: string, model: string): string {
  switch (language) {
    case "curl":
      return [
        `curl ${baseUrl}/chat/completions \\`,
        `  -H "Authorization: Bearer $${KEY_VARIABLE}" \\`,
        `  -H "Content-Type: application/json" \\`,
        `  -d '{"model": "${model}", "messages": [{"role": "user", "content": "Hello!"}]}'`,
      ].join("\n");
    case "python":
      return [
        "import os",
        "from openai import OpenAI",
        "",
        `client = OpenAI(base_url="${baseUrl}", api_key=os.environ["${KEY_VARIABLE}"])`,
        "reply = client.chat.completions.create(",
        `    model="${model}",`,
        '    messages=[{"role": "user", "content": "Hello!"}],',
        ")",
        "print(reply.choices[0].message.content)",
      ].join("\n");
    case "javascript":
      return [
        'import OpenAI from "openai";',
        "",
        `const client = new OpenAI({ baseURL: "${baseUrl}", apiKey: process.env.${KEY_VARIABLE} });`,
        "const reply = await client.chat.completions.create({",
        `  model: "${model}",`,
        '  messages: [{ role: "user", content: "Hello!" }],',
        "});",
        "console.log(reply.choices[0].message.content);",
      ].join("\n");
  }
}
