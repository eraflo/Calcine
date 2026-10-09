/**
 * The system Calcine runs on: Windows (WebView2) or Linux (WebKitGTK). Both
 * say so in the user agent.
 */
export const isWindows = typeof navigator === "undefined" || !navigator.userAgent.includes("Linux");
