import { defineConfig } from "bumpp";

export default defineConfig({
	release: "prompt",
	commit: true,
	tag: true,
	push: true,
	files: [
		"package.json",
		"src-tauri/tauri.conf.json",
		"src-tauri/Cargo.toml"
	]
});
