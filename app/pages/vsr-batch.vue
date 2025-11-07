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
						<label class="text-sm font-semibold text-(--ui-text-muted)">
							LDAP / Radius 二选一
						</label>
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
						<div class="grid gap-4 md:grid-cols-[220px_1fr] items-center">
							<label class="text-sm font-medium text-(--ui-text-muted)">
								工作表
							</label>
							<USelect
								v-model="excelState.selectedSheet"
								:options="excelState.analysis.sheetNames"
								placeholder="选择工作表"
								class="w-full"
							/>
						</div>

						<div class="space-y-3">
							<div class="flex flex-wrap items-center justify-between gap-3">
								<h4 class="text-base font-semibold">
									列映射
								</h4>
								<UButton
									variant="soft"
									size="sm"
									icon="i-lucide-refresh-cw"
									@click="resetColumnMapping"
								>
									重置为推荐
								</UButton>
							</div>
							<div class="flex flex-wrap gap-4">
								<div
									v-for="field in fieldDefinitions"
									:key="field.key"
									class="flex flex-col gap-2 min-w-[200px] flex-1"
								>
									<div class="flex items-center justify-between gap-2">
										<span class="text-xs font-semibold text-(--ui-text-muted)">
											{{ field.label }}
											<span v-if="field.required" class="text-red-500">*</span>
										</span>
										<div v-if="isFieldMapped(field.key)" class="flex items-center gap-1 text-xs text-green-600">
											<Icon name="i-lucide-check" class="size-3" />
											<span>已映射</span>
										</div>
										<div v-else class="flex items-center gap-1 text-xs text-gray-400">
											<Icon name="i-lucide-circle" class="size-3" />
											<span>未映射</span>
										</div>
									</div>
									<USelect
										v-model="excelState.columnMapping[field.key]"
										:items="columnOptions"
										placeholder="选择 Excel 列"
										class="w-full"
										size="sm"
										:popper="{ strategy: 'fixed', placement: 'bottom-start' }"
									/>
								</div>
							</div>
						</div>

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
							:disabled="!excelState.analysis || isColumnMappingIncomplete"
							:loading="convertLoading"
							icon="i-lucide-check"
							@click="convertExcelData"
						>
							解析 Excel 数据
						</UButton>
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
					<div
						v-if="convertErrors.length"
						class="rounded-md border border-yellow-300 bg-yellow-50 px-4 py-3 text-sm text-yellow-800"
					>
						<p class="font-medium">
							检测到 {{ convertErrors.length }} 条告警：
						</p>
						<ul class="list-disc pl-5 space-y-1">
							<li v-for="(error, index) in convertErrors" :key="`vsr-convert-error-${index}`">
								{{ error }}
							</li>
						</ul>
					</div>

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
							<UButton
								variant="outline"
								size="sm"
								icon="i-lucide-refresh-cw"
								@click="regeneratePreview"
							>
								刷新预览
							</UButton>
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
							icon="i-lucide-cpu"
							:loading="generationLoading"
							:disabled="!vsrEntries.length"
							@click="generateConfigs"
						>
							生成配置
						</UButton>
						<UButton
							variant="outline"
							icon="i-lucide-save"
							:disabled="!generatedConfigs.length"
							@click="exportConfigs"
						>
							导出为 Excel
						</UButton>
					</div>

					<div v-if="generatedConfigs.length" class="space-y-3">
						<p class="text-sm text-(--ui-text-muted)">
							预览前 {{ Math.min(generatedConfigs.length, 5) }} 台设备：
						</p>
						<UTable
							:columns="configPreviewColumns"
							:data="configPreviewRows"
							class="w-full"
						>
							<template #config-data="{ row }">
								<pre class="bg-(--ui-bg-muted) rounded p-3 text-xs whitespace-pre-wrap">{{ row.config }}</pre>
							</template>
						</UTable>
					</div>
				</div>
			</UCard>
		</div>
	</LayoutTile>
</template>

<script lang="ts" setup>
import { computed, reactive, ref, watch } from "vue";
import LayoutTile from "~/components/Layout/Tile.vue";
import type { ConvertResponse, ExcelAnalysis, VsrEntry, VsrGeneratedConfig } from "~/types/vsr-batch";
import { extractErrorMessage } from "~/utils/error";

const fieldDefinitions = [
	{ key: "device_name", label: "设备名称", required: true },
	{ key: "ip", label: "设备 IP", required: true },
	{ key: "gateway", label: "网关", required: true },
	{ key: "pool_cidr", label: "地址池 CIDR", required: true },
	{ key: "vsr_username", label: "管理员账号", required: false },
	{ key: "vsr_password", label: "管理员密码", required: false },
	{ key: "monitor_username", label: "监控账号", required: false },
	{ key: "monitor_password", label: "监控密码", required: false },
	{ key: "ppp_username", label: "PPP 用户", required: false },
	{ key: "ppp_password", label: "PPP 密码", required: false },
	{ key: "start_ip", label: "地址池起始 IP", required: false },
	{ key: "end_ip", label: "地址池结束 IP", required: false },
	{ key: "pool_ip_gateway", label: "地址池网关", required: false },
	{ key: "ldap_server_ip", label: "LDAP 服务器", required: false },
	{ key: "ldap_login_dn", label: "LDAP Login DN", required: false },
	{ key: "ldap_search_base_dn", label: "LDAP Search Base DN", required: false },
	{ key: "ldap_password", label: "LDAP 密码", required: false },
	{ key: "radius_ip", label: "Radius 服务器", required: false },
	{ key: "radius_password", label: "Radius 密码", required: false }
] as const;

const UNMAPPED_VALUE = "__unmapped__";

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

const excelState = reactive({
	filePath: "",
	analysis: null as ExcelAnalysis | null,
	selectedSheet: "",
	columnMapping: fieldDefinitions.reduce<Record<string, string>>((acc, field) => {
		acc[field.key] = UNMAPPED_VALUE;
		return acc;
	}, {}),
	previewRows: [] as string[][],
	isLoading: false
});

const convertLoading = ref(false);
const generationLoading = ref(false);
const convertErrors = ref<string[]>([]);
const vsrEntries = ref<VsrEntry[]>([]);
const generatedConfigs = ref<VsrGeneratedConfig[]>([]);

const excelColumns = computed(() => excelState.analysis?.columns ?? []);
const previewRows = computed(() => excelState.previewRows.slice(0, 10));

const columnOptions = computed(() => [
	{ label: "未选择", value: UNMAPPED_VALUE },
	...excelColumns.value.map((col) => ({ label: col || "(空列)", value: col || UNMAPPED_VALUE }))
]);

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

const isColumnMappingIncomplete = computed(() =>
	fieldDefinitions
		.filter((field) => field.required)
		.some((field) => {
			const value = excelState.columnMapping[field.key];
			return !value || value === UNMAPPED_VALUE;
		})
);

const entryPreviewColumns = [
	{ id: "deviceName", header: "设备", accessorKey: "deviceName" },
	{ id: "ip", header: "IP", accessorKey: "ip" },
	{ id: "gateway", header: "网关", accessorKey: "gateway" },
	{ id: "poolCidr", header: "地址池 CIDR", accessorKey: "poolCidr" },
	{ id: "startIp", header: "地址池起始", accessorKey: "startIp" },
	{ id: "endIp", header: "地址池结束", accessorKey: "endIp" }
];

const entryPreviewRows = computed(() => vsrEntries.value.slice(0, 10));

const configPreviewColumns = [
	{ id: "deviceName", header: "设备", accessorKey: "deviceName" },
	{ id: "config", header: "配置", accessorKey: "config" }
];

const configPreviewRows = computed(() => generatedConfigs.value.slice(0, 5));

watch(
	() => excelState.selectedSheet,
	(sheet, prev) => {
		if (!excelState.analysis || !excelState.filePath || sheet === prev || !sheet) return;
		void analyzeExcel(sheet);
	}
);

function isFieldMapped(key: string) {
	const value = excelState.columnMapping[key];
	return Boolean(value && value !== UNMAPPED_VALUE);
}

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
		excelState.columnMapping = fieldDefinitions.reduce<Record<string, string>>((acc, field) => {
			acc[field.key] = analysis.suggestedMapping[field.key] ?? UNMAPPED_VALUE;
			return acc;
		}, {});
		excelState.previewRows = analysis.previewRows;
		vsrEntries.value = [];
		generatedConfigs.value = [];
		convertErrors.value = [];

		toast.add({
			title: "Excel 加载成功",
			description: `检测到 ${analysis.totalRows} 行数据，完成自动解析`,
			color: "success"
		});
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

async function convertExcelData() {
	if (!excelState.analysis || isColumnMappingIncomplete.value) {
		toast.add({
			title: "列映射不完整",
			description: "请为所有必填字段选择对应的列。",
			color: "warning"
		});
		return;
	}

	convertLoading.value = true;
	try {
		const selectedSheet = excelState.selectedSheet && excelState.selectedSheet.trim().length > 0
			? excelState.selectedSheet
			: undefined;
		const normalizedMapping = Object.fromEntries(
			fieldDefinitions.map((field) => [
				field.key,
				excelState.columnMapping[field.key] === UNMAPPED_VALUE ? "" : excelState.columnMapping[field.key]
			])
		);

		const response = await useTauriCoreInvoke<ConvertResponse>("convert_vsr_entries", {
			request: {
				filePath: excelState.filePath,
				sheetName: selectedSheet,
				headerRowIndex: excelState.analysis.headerRowIndex,
				columnMapping: normalizedMapping
			}
		});

		vsrEntries.value = response.entries;
		convertErrors.value = response.errors;
		generatedConfigs.value = [];

		toast.add({
			title: "Excel 数据解析成功",
			description: `有效记录 ${response.entries.length} 条`,
			color: "success"
		});
	} catch (error) {
		convertErrors.value = [extractErrorMessage(error)];
		vsrEntries.value = [];
		toast.add({
			title: "解析失败",
			description: extractErrorMessage(error),
			color: "error"
		});
	} finally {
		convertLoading.value = false;
	}
}

function clearExcelContext() {
	excelState.analysis = null;
	excelState.filePath = "";
	excelState.selectedSheet = "";
	excelState.columnMapping = fieldDefinitions.reduce<Record<string, string>>((acc, field) => {
		acc[field.key] = UNMAPPED_VALUE;
		return acc;
	}, {});
	excelState.previewRows = [];
	vsrEntries.value = [];
	generatedConfigs.value = [];
	convertErrors.value = [];
}

function resetColumnMapping() {
	if (!excelState.analysis) return;
	excelState.columnMapping = fieldDefinitions.reduce<Record<string, string>>((acc, field) => {
		acc[field.key] = excelState.analysis?.suggestedMapping[field.key] ?? UNMAPPED_VALUE;
		return acc;
	}, {});
}

function regeneratePreview() {
	vsrEntries.value = [...vsrEntries.value];
}

async function generateConfigs() {
	if (!vsrEntries.value.length) {
		toast.add({
			title: "无可生成的数据",
			description: "请先解析 Excel 数据。",
			color: "warning"
		});
		return;
	}

	generationLoading.value = true;
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
		toast.add({
			title: "配置生成完成",
			description: `共生成 ${configs.length} 台设备配置`,
			color: "success"
		});
	} catch (error) {
		toast.add({
			title: "生成失败",
			description: extractErrorMessage(error),
			color: "error"
		});
	} finally {
		generationLoading.value = false;
	}
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
	category: "firewall"
});
</script>
