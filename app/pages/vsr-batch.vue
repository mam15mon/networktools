<template>
	<LayoutTile
		title="VSR 批量配置生成"
		description="基于 Excel 模板批量生成 VSR 配置脚本，支持本地、LDAP、Radius 可选段落。"
	>
		<div class="space-y-8">
		<UCard class="bg-(--ui-bg)">
			<template #header>
				<div class="flex items-center gap-2">
					<Icon name="i-lucide-shield-check" class="size-5" />
					<h3 class="text-lg font-semibold">
						认证策略
					</h3>
				</div>
				</template>
				<div class="space-y-4">
					<div class="flex flex-wrap items-center gap-3">
						<UCheckbox v-model="enableLocalAuth" label="启用本地认证 (PPP/管理员/监控)" />
					</div>
				<div class="space-y-2">
					<p class="text-sm font-semibold text-(--ui-text-muted)">
						LDAP / Radius 二选一
					</p>
						<URadioGroup
							v-model="remoteAuth"
							:items="remoteAuthOptions"
							orientation="horizontal"
							:disabled="!enableLocalAuth"
							class="flex flex-wrap gap-4"
						/>
						<p class="text-xs text-(--ui-text-muted)">
							若仅需本地认证，请勾选“启用本地认证”并保持下方单选为“仅本地”。
						</p>
					</div>
				</div>
			</UCard>

		<UCard class="bg-(--ui-bg)">
			<template #header>
				<div class="flex items-center gap-2">
					<Icon name="i-lucide-file-spreadsheet" class="size-5" />
					<h3 class="text-lg font-semibold">
						Excel 数据
					</h3>
				</div>
				</template>
				<div class="space-y-6">
					<div class="space-y-3">
						<div class="flex flex-wrap items-center gap-3">
							<UButton :loading="excelState.isLoading" icon="i-lucide-folder-open" @click="handleSelectExcel">
								选择 Excel 文件
							</UButton>
							<UButton variant="outline" icon="i-lucide-download" @click="handleExportTemplate">
								导出模板
							</UButton>
						</div>
						<div v-if="excelState.filePath" class="p-3 bg-(--ui-bg-muted) rounded-md">
							<p class="text-sm text-(--ui-text-muted) break-all">
								<span class="font-medium">当前文件：</span>{{ excelState.filePath }}
							</p>
						</div>
					</div>

				<div v-if="excelState.analysis" class="space-y-4">
				<div class="space-y-2">
					<div class="flex items-center justify-between">
						<h4 class="text-base font-semibold">
							数据预览（{{ excelState.analysis.totalRows }} 行，展示前 {{ previewRows.length }} 行）
						</h4>
							</div>
							<UTable
								:columns="excelPreviewColumns"
								:data="excelPreviewRows"
								class="w-full"
							/>
						</div>
					</div>

				<div class="flex flex-wrap gap-3">
					<UButton
						variant="outline"
						size="sm"
						:disabled="!excelState.analysis"
						icon="i-lucide-eraser"
						@click="clearExcelContext"
					>
						清除已加载数据
					</UButton>
				</div>
				</div>
			</UCard>

			<UCard v-if="convertErrors.length || vsrEntries.length" class="bg-(--ui-bg)">
				<template #header>
					<div class="flex items-center gap-2">
						<Icon name="i-lucide-table" class="size-5" />
						<h3 class="text-lg font-semibold">
							数据校验结果
						</h3>
					</div>
				</template>
				<div class="space-y-4">
				<UAlert
					v-if="convertErrors.length"
					variant="warning"
					icon="i-lucide-alert-triangle"
					:title="`检测到 ${convertErrors.length} 条告警：`"
				>
					<ul class="list-disc pl-5 space-y-1">
						<li v-for="(error, index) in convertErrors" :key="`vsr-convert-error-${index}`">
							{{ error }}
						</li>
					</ul>
				</UAlert>

					<div v-if="vsrEntries.length" class="space-y-3">
						<div class="flex flex-wrap items-center justify-between gap-3">
							<div>
								<p class="text-base font-semibold">
									已校验通过 {{ vsrEntries.length }} 条记录
								</p>
								<p class="text-sm text-(--ui-text-muted)">
									预览前 {{ Math.min(vsrEntries.length, 10) }} 条，生成配置时会全部使用。
								</p>
							</div>
							<div class="text-sm text-(--ui-text-muted)">
								上传完成后自动生成配置，可直接下载。
							</div>
						</div>
						<UTable
							:columns="entryPreviewColumns"
							:data="entryPreviewRows"
							class="w-full"
						/>
					</div>
				</div>
			</UCard>

		<UCard class="bg-(--ui-bg)">
			<template #header>
				<div class="flex items-center gap-2">
					<Icon name="i-lucide-terminal" class="size-5" />
					<h3 class="text-lg font-semibold">
						配置输出
					</h3>
				</div>
				</template>
				<div class="space-y-4">
					<div class="flex flex-wrap gap-3">
						<UButton
							variant="outline"
							icon="i-lucide-save"
							:loading="convertLoading || generationLoading"
							:disabled="!generatedConfigs.length || convertLoading || generationLoading"
							@click="exportConfigs"
						>
							导出为 Excel
						</UButton>
					</div>

		<div v-if="generatedConfigs.length" class="space-y-3">
			<p class="text-sm text-(--ui-text-muted)">
				预览前 {{ Math.min(generatedConfigs.length, 5) }} 台设备：
			</p>
			<ConfigDeviceCards
				:items="configPreviewItems"
				:limit="5"
				empty-message="生成结果将显示在此处。"
			/>
		</div>
				<div v-else class="rounded-lg border border-dashed border-(--ui-border) p-4 text-sm text-(--ui-text-muted)">
					生成结果将显示在此处。
				</div>
				</div>
			</UCard>
		</div>
	</LayoutTile>
</template>

<script lang="ts" setup>
import { computed, reactive, ref, watch } from "vue";
import LayoutTile from "~/components/Layout/Tile.vue";
import ConfigDeviceCards from "~/components/Config/DeviceCards.vue";
import type { ConvertResponse, ExcelAnalysis, VsrEntry, VsrGeneratedConfig } from "~/types/vsr-batch";
import { extractErrorMessage } from "~/utils/error";

const toast = useToast();

const enableLocalAuth = ref(true);
const remoteAuth = ref<"none" | "ldap" | "radius">("none");
const remoteAuthOptions = [
	{ label: "仅本地", value: "none" },
	{ label: "LDAP", value: "ldap" },
	{ label: "Radius", value: "radius" }
];

watch(enableLocalAuth, (value) => {
	if (!value && remoteAuth.value !== "none") {
		remoteAuth.value = "none";
	}
});

watch([enableLocalAuth, remoteAuth], () => {
	if (!vsrEntries.value.length) {
		return;
	}
	void generateConfigs({ silentSuccess: true, silentEmpty: true });
});

const excelState = reactive({
	filePath: "",
	analysis: null as ExcelAnalysis | null,
	selectedSheet: "",
	previewRows: [] as string[][],
	isLoading: false
});

const convertLoading = ref(false);
const generationLoading = ref(false);
const convertErrors = ref<string[]>([]);
const vsrEntries = ref<VsrEntry[]>([]);
const generatedConfigs = ref<VsrGeneratedConfig[]>([]);
let generationTask: Promise<boolean> | null = null;

const excelColumns = computed(() => excelState.analysis?.columns ?? []);
const previewRows = computed(() => excelState.previewRows.slice(0, 10));

const excelPreviewColumns = computed(() =>
	excelColumns.value.map((column, index) => ({
		id: `col_${index}`,
		accessorKey: `col_${index}`,
		header: column || `列 ${index + 1}`
	}))
);

const excelPreviewRows = computed(() => {
	return previewRows.value.map((row) => {
		const obj: Record<string, string> = {};
		row.forEach((cell, index) => {
			obj[`col_${index}`] = cell;
		});
		return obj;
	});
});

const entryPreviewColumns = [
	{ id: "deviceName", header: "设备", accessorKey: "deviceName" },
	{ id: "ip", header: "IP", accessorKey: "ip" },
	{ id: "gateway", header: "网关", accessorKey: "gateway" },
	{ id: "poolCidr", header: "地址池 CIDR", accessorKey: "poolCidr" },
	{ id: "startIp", header: "地址池起始", accessorKey: "startIp" },
	{ id: "endIp", header: "地址池结束", accessorKey: "endIp" }
];

const entryPreviewRows = computed(() => vsrEntries.value.slice(0, 10));

const configPreviewItems = computed(() =>
	generatedConfigs.value.map((item) => ({
		label: item.deviceName,
		config: item.config
	}))
);

async function handleSelectExcel() {
	try {
		const { open } = await import("@tauri-apps/plugin-dialog");
		const selected = await open({ multiple: false, filters: [{ name: "Excel", extensions: ["xlsx", "xls"] }] });
		if (!selected) return;
		const path = Array.isArray(selected) ? selected[0] : selected;
		if (!path) return;
		excelState.filePath = path;
		await analyzeExcel();
	} catch (error) {
		toast.add({
			title: "选择文件失败",
			description: extractErrorMessage(error),
			color: "error"
		});
	}
}

async function analyzeExcel(sheetName?: string) {
	if (!excelState.filePath) return;
	excelState.isLoading = true;
	try {
		const targetSheet = sheetName ?? excelState.selectedSheet;
		const normalizedSheet = targetSheet && targetSheet.trim().length > 0 ? targetSheet : undefined;

		const analysis = await useTauriCoreInvoke<ExcelAnalysis>("process_vsr_excel", {
			request: {
				filePath: excelState.filePath,
				sheetName: normalizedSheet
			}
		});

		excelState.analysis = analysis;
		excelState.selectedSheet = analysis.selectedSheet;
		excelState.previewRows = analysis.previewRows;
		vsrEntries.value = [];
		generatedConfigs.value = [];
		convertErrors.value = [];

		await runAutoPipeline();
	} catch (error) {
		excelState.analysis = null;
		excelState.previewRows = [];
		convertErrors.value = [extractErrorMessage(error)];
		toast.add({
			title: "加载 Excel 失败",
			description: extractErrorMessage(error),
			color: "error"
		});
	} finally {
		excelState.isLoading = false;
	}
}

async function runAutoPipeline() {
	const convertOk = await convertExcelData();
	if (!convertOk || !vsrEntries.value.length) {
		return;
	}
	await generateConfigs({ silentSuccess: true, silentEmpty: true });
}

async function convertExcelData(options: { silentSuccess?: boolean } = {}): Promise<boolean> {
	if (!excelState.analysis) {
		return false;
	}

	convertLoading.value = true;
	try {
		const selectedSheet = excelState.selectedSheet && excelState.selectedSheet.trim().length > 0
			? excelState.selectedSheet
			: undefined;

		const response = await useTauriCoreInvoke<ConvertResponse>("convert_vsr_entries", {
			request: {
				filePath: excelState.filePath,
				sheetName: selectedSheet,
				headerRowIndex: excelState.analysis.headerRowIndex
			}
		});

		vsrEntries.value = response.entries;
		convertErrors.value = response.errors;
		generatedConfigs.value = [];

		if (!options.silentSuccess) {
			toast.add({
				title: "Excel 数据解析成功",
				description: `有效记录 ${response.entries.length} 条`,
				color: "success"
			});
		}
		return true;
	} catch (error) {
		convertErrors.value = [extractErrorMessage(error)];
		vsrEntries.value = [];
		generatedConfigs.value = [];
		toast.add({
			title: "解析失败",
			description: extractErrorMessage(error),
			color: "error"
		});
		return false;
	} finally {
		convertLoading.value = false;
	}
}

function clearExcelContext() {
	excelState.analysis = null;
	excelState.filePath = "";
	excelState.selectedSheet = "";
	excelState.previewRows = [];
	vsrEntries.value = [];
	generatedConfigs.value = [];
	convertErrors.value = [];
}

async function generateConfigs(options: { silentSuccess?: boolean; silentEmpty?: boolean } = {}): Promise<boolean> {
	if (!vsrEntries.value.length) {
		if (!options.silentEmpty) {
			toast.add({
				title: "无可生成的数据",
				description: "请先解析 Excel 数据。",
				color: "warning"
			});
		}
		return false;
	}

	if (generationTask) {
		await generationTask;
	}

	generationLoading.value = true;
	const task = (async () => {
		try {
			const configs = await useTauriCoreInvoke<VsrGeneratedConfig[]>("generate_vsr_configs", {
				request: {
					entries: vsrEntries.value,
					includeLocal: enableLocalAuth.value,
					includeLdap: remoteAuth.value === "ldap",
					includeRadius: remoteAuth.value === "radius"
				}
			});
			generatedConfigs.value = configs;
			if (!options.silentSuccess) {
				toast.add({
					title: "配置生成完成",
					description: `共生成 ${configs.length} 台设备配置`,
					color: "success"
				});
			}
			return true;
		} catch (error) {
			toast.add({
				title: "生成失败",
				description: extractErrorMessage(error),
				color: "error"
			});
			return false;
		} finally {
			generationLoading.value = false;
		}
	})();

	generationTask = task;
	const result = await task;
	if (generationTask === task) {
		generationTask = null;
	}
	return result;
}

async function exportConfigs() {
	if (!generatedConfigs.value.length) return;
	try {
		const { save } = await import("@tauri-apps/plugin-dialog");
		const path = await save({
			defaultPath: "vsr_configs.xlsx",
			filters: [{ name: "Excel", extensions: ["xlsx"] }]
		});
		if (!path) return;
		await useTauriCoreInvoke("export_vsr_configs", {
			path,
			configs: generatedConfigs.value
		});
		toast.add({
			title: "导出成功",
			description: `文件已保存到 ${path}`,
			color: "success"
		});
	} catch (error) {
		toast.add({
			title: "导出失败",
			description: extractErrorMessage(error),
			color: "error"
		});
	}
}

async function handleExportTemplate() {
	try {
		const { save } = await import("@tauri-apps/plugin-dialog");
		const path = await save({
			defaultPath: "vsr_template.xlsx",
			filters: [{ name: "Excel", extensions: ["xlsx"] }]
		});
		if (!path) return;
		await useTauriCoreInvoke("export_vsr_template", {
			request: {
				path,
				includeLocal: enableLocalAuth.value,
				includeLdap: remoteAuth.value === "ldap",
				includeRadius: remoteAuth.value === "radius"
			}
		});
		toast.add({
			title: "模板导出成功",
			description: `模板已保存到 ${path}`,
			color: "success"
		});
	} catch (error) {
		toast.add({
			title: "导出失败",
			description: extractErrorMessage(error),
			color: "error"
		});
	}
}

definePageMeta({
	name: "VSR 批量配置",
	icon: "lucide:server-cog",
	description: "批量生成 VSR 设备配置脚本",
	category: "other"
});
</script>
