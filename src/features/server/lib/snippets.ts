export type SnippetLanguage =
  | "curl"
  | "python"
  | "javascript"
  | "langchain"
  | "continue"
  | "openwebui";

export const SNIPPET_LANGUAGES: { id: SnippetLanguage; label: string }[] = [
  { id: "curl", label: "curl" },
  { id: "python", label: "Python" },
  { id: "javascript", label: "JavaScript" },
  { id: "langchain", label: "LangChain" },
  { id: "continue", label: "Continue" },
  { id: "openwebui", label: "Open WebUI" },
];

/** Comments in the setup snippets, in the UI language. */
export type SnippetNotes = {
  /** Where API keys are created, e.g. "Server › API keys". */
  keys: string;
  /** Where Open WebUI takes an OpenAI connection. */
  openWebUiSettings: string;
  /** Open WebUI must run on this PC, not in Docker. */
  openWebUiLocal: string;
};

const ENGLISH_NOTES: SnippetNotes = {
  keys: "Server › API keys",
  openWebUiSettings: "Admin Panel › Settings › Connections › OpenAI API › Add connection",
  openWebUiLocal:
    "Run Open WebUI on this PC (pip install open-webui): Calcine only listens on this PC, so Docker containers can't reach it.",
};

/** Environment variable the snippets read the API key from. */
export const KEY_VARIABLE = "CALCINE_API_KEY";

/** Copy-paste examples for calling the local API from another app. */
export function snippet(
  language: SnippetLanguage,
  baseUrl: string,
  model: string,
  notes: SnippetNotes = ENGLISH_NOTES,
): string {
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
    case "langchain":
      return [
        "import os",
        "from langchain_openai import ChatOpenAI",
        "",
        "llm = ChatOpenAI(",
        `    base_url="${baseUrl}",`,
        `    api_key=os.environ["${KEY_VARIABLE}"],`,
        `    model="${model}",`,
        ")",
        'print(llm.invoke("Hello!").content)',
      ].join("\n");
    case "continue":
      return [
        "# ~/.continue/config.yaml",
        "models:",
        `  - name: ${model.split("/").pop()} (Calcine)`,
        "    provider: openai",
        `    model: ${model}`,
        `    apiBase: ${baseUrl}`,
        `    apiKey: \${{ secrets.${KEY_VARIABLE} }}  # ${notes.keys}`,
      ].join("\n");
    case "openwebui":
      return [
        `# Open WebUI › ${notes.openWebUiSettings}`,
        `URL: ${baseUrl}`,
        `Key: $${KEY_VARIABLE}  # ${notes.keys}`,
        "",
        `# ${notes.openWebUiLocal}`,
      ].join("\n");
  }
}
