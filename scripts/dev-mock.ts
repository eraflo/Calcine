/** Run `tauri dev` with the in-memory mock backend (no GenieX needed). */
const child = Bun.spawn(["bun", "x", "tauri", "dev"], {
  env: { ...process.env, CALCINE_BACKEND: "mock" },
  stdio: ["inherit", "inherit", "inherit"],
});
process.exit(await child.exited);
