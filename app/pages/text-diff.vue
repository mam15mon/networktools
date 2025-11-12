<template>
	<LayoutTile title="文本对比" description="支持粘贴、选择或拖拽文件，双栏同步展示差异并导出结果。">
		<div class="space-y-8 w-full max-w-screen-2xl mx-auto">
			<div class="grid gap-6 lg:grid-cols-2">
				<UCard class="bg-(--ui-bg)">
					<template #header>
						<div class="flex items-center justify-between">
							<div class="space-y-1">
								<p class="text-sm font-semibold text-(--ui-text)">
									文本 A
								</p>
								<p class="text-xs text-(--ui-text-muted)">
									{{ leftMeta || "手动粘贴" }}
								</p>
							</div>
							<div class="flex items-center gap-2">
								<UButton size="xs" variant="ghost" icon="i-lucide-upload" @click="triggerFile('left')">
									导入文件
								</UButton>
								<UButton size="xs" variant="ghost" icon="i-lucide-eraser" @click="clearText('left')">
									清空
								</UButton>
							</div>
						</div>
					</template>
					<div
						class="relative"
						@dragover.prevent
						@drop.prevent="handleDrop($event, 'left')"
					>
						<UTextarea
							v-model="leftText"
							:rows="14"
							placeholder="粘贴文本或拖入文件..."
							class="font-mono"
						/>
						<p class="mt-2 text-xs text-(--ui-text-muted)">
							支持拖拽文件到此区域自动读取。
						</p>
					</div>
				</UCard>

				<UCard class="bg-(--ui-bg)">
					<template #header>
						<div class="flex items-center justify-between">
							<div class="space-y-1">
								<p class="text-sm font-semibold text-(--ui-text)">
									文本 B
								</p>
								<p class="text-xs text-(--ui-text-muted)">
									{{ rightMeta || "手动粘贴" }}
								</p>
							</div>
							<div class="flex items-center gap-2">
								<UButton size="xs" variant="ghost" icon="i-lucide-upload" @click="triggerFile('right')">
									导入文件
								</UButton>
								<UButton size="xs" variant="ghost" icon="i-lucide-eraser" @click="clearText('right')">
									清空
								</UButton>
							</div>
						</div>
					</template>
					<div
						class="relative"
						@dragover.prevent
						@drop.prevent="handleDrop($event, 'right')"
					>
						<UTextarea
							v-model="rightText"
							:rows="14"
							placeholder="粘贴文本或拖入文件..."
							class="font-mono"
						/>
						<p class="mt-2 text-xs text-(--ui-text-muted)">
							支持拖拽文件到此区域自动读取。
						</p>
					</div>
				</UCard>
			</div>

			<div class="flex flex-wrap gap-3">
				<UButton
					icon="i-lucide-scan-line"
					:loading="isComparing"
					@click="generateDiff"
				>
					生成对比
				</UButton>
				<UButton
					variant="outline"
					icon="i-lucide-file-down"
					:disabled="!diffText"
					@click="downloadTextDiff"
				>
					导出文本 diff
				</UButton>
				<UButton
					variant="outline"
					icon="i-lucide-file-type"
					:disabled="!diffHtml"
					@click="downloadHtmlDiff"
				>
					导出 HTML 对比
				</UButton>
			</div>

			<UAlert v-if="!diffHtml" color="neutral" variant="subtle">
				<template #title>
					操作说明
				</template>
				<template #description>
					粘贴或导入两份文本，点击“生成对比”即可在下方查看差异；支持拖拽文件、导出文本/HTML diff。
				</template>
			</UAlert>

			<div v-if="diffHtml" class="overflow-auto border border-(--ui-border) rounded-lg">
				<div class="diff2html" v-html="diffHtml" />
			</div>
		</div>
		<input ref="leftFileInput" type="file" class="hidden" @change="handleFileSelect($event, 'left')">
		<input ref="rightFileInput" type="file" class="hidden" @change="handleFileSelect($event, 'right')">
	</LayoutTile>
</template>

<script setup lang="ts">
	import { createTwoFilesPatch } from "diff";
	import { Diff2Html } from "diff2html";
	import { ref } from "vue";
	import LayoutTile from "~/components/Layout/Tile.vue";
	import "diff2html/bundles/css/diff2html.min.css";

	const leftText = ref("");
	const rightText = ref("");
	const leftMeta = ref("");
	const rightMeta = ref("");
	const diffHtml = ref("");
	const diffText = ref("");
	const isComparing = ref(false);
	const toast = useToast();
	const leftFileInput = ref<HTMLInputElement>();
	const rightFileInput = ref<HTMLInputElement>();

	function triggerFile(side: "left" | "right") {
		(side === "left" ? leftFileInput.value : rightFileInput.value)?.click();
	}

	function clearText(side: "left" | "right") {
		if (side === "left") {
			leftText.value = "";
			leftMeta.value = "";
		} else {
			rightText.value = "";
			rightMeta.value = "";
		}
		diffHtml.value = "";
		diffText.value = "";
	}

	async function handleFileSelect(event: Event, side: "left" | "right") {
		const target = event.target as HTMLInputElement;
		const file = target.files?.[0];
		if (!file) return;
		const text = await file.text();
		if (side === "left") {
			leftText.value = text;
			leftMeta.value = file.name;
		} else {
			rightText.value = text;
			rightMeta.value = file.name;
		}
		(target.value as unknown as string) = "";
	}

	async function handleDrop(event: DragEvent, side: "left" | "right") {
		const file = event.dataTransfer?.files?.[0];
		if (!file) return;
		if (!file.type.startsWith("text") && file.type !== "") {
			toast.add({ title: "暂不支持的文件类型", color: "warning" });
			return;
		}
		const text = await file.text();
		if (side === "left") {
			leftText.value = text;
			leftMeta.value = file.name;
		} else {
			rightText.value = text;
			rightMeta.value = file.name;
		}
	}

	function generateDiff() {
		if (!leftText.value && !rightText.value) {
			toast.add({ title: "请输入内容", description: "至少提供一份文本", color: "warning" });
			return;
		}
		isComparing.value = true;
		try {
			const patch = createTwoFilesPatch(
				leftMeta.value || "文本A",
				rightMeta.value || "文本B",
				leftText.value,
				rightText.value,
				"",
				"",
				{
					context: 3
				}
			);
			diffText.value = patch;
			diffHtml.value = Diff2Html.html(patch, {
				drawFileList: false,
				matching: "lines",
				outputFormat: "side-by-side",
				diffStyle: "word"
			});
		} finally {
			isComparing.value = false;
		}
	}

	function downloadTextDiff() {
		if (!diffText.value) return;
		downloadBlob(diffText.value, "text-diff.patch", "text/plain;charset=utf-8");
	}

	function downloadHtmlDiff() {
		if (!diffHtml.value) return;
		const html = `<!DOCTYPE html><html lang="zh"><head><meta charset="UTF-8" /><title>文本对比</title><link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/diff2html/bundles/css/diff2html.min.css" /></head><body><div class="diff2html">${diffHtml.value}</div></body></html>`;
		downloadBlob(html, "text-diff.html", "text/html;charset=utf-8");
	}

	function downloadBlob(content: string, filename: string, type: string) {
		const blob = new Blob([content], { type });
		const url = URL.createObjectURL(blob);
		const a = document.createElement("a");
		a.href = url;
		a.download = filename;
		a.click();
		URL.revokeObjectURL(url);
	}
</script>
