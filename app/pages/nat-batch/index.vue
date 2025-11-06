<template>
	<LayoutTile
		title="NAT 批量配置生成"
		description="从 Excel 或手动输入批量生成防火墙 NAT 配置命令，集成弹性 IP 映射与运营商数据维护。"
	>
		<div class="space-y-8">
			<UCard class="bg-(--ui-bg)">
				<template #header>
					<div class="flex items-center gap-2">
						<Icon name="i-lucide-merge" class="size-5" />
						<h3 class="text-lg font-semibold">
							数据来源
						</h3>
					</div>
				</template>
				<div class="flex flex-wrap items-center gap-6">
					<URadioGroup v-model="mode" :items="modeOptions" class="flex gap-6 flex-wrap" />
					<p class="text-sm text-(--ui-text-muted)">
						选择 Excel 导入或手动录入需求，后续步骤将根据模式自动调整。
					</p>
				</div>
			</UCard>

			<div v-if="mode === 'excel'" class="space-y-6">
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
						<div class="flex flex-wrap items-center gap-3">
							<UButton :loading="excelState.isLoading" icon="i-lucide-folder-open" @click="handleSelectExcel">
								选择 Excel 文件
							</UButton>
							<UButton variant="outline" icon="i-lucide-download" @click="handleExportTemplate">
								导出模板
							</UButton>
							<p v-if="excelState.filePath" class="text-sm text-(--ui-text-muted) break-all">
								当前文件：{{ excelState.filePath }}
							</p>
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
								<div class="flex items-center justify-between">
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
								<div class="grid gap-4 lg:grid-cols-2">
									<div
										v-for="field in requiredFields"
										:key="field"
										class="space-y-1.5"
									>
										<label class="text-xs font-semibold text-(--ui-text-muted)">
											{{ field }}
										</label>
										<select
											v-model="excelState.columnMapping[field]"
											class="select-base"
										>
											<option value="">
												未选择
											</option>
											<option
												v-for="column in excelColumns"
												:key="column || `col-${field}`"
												:value="column"
											>
												{{ column || "(空列)" }}
											</option>
										</select>
									</div>
								</div>
							</div>

							<div class="space-y-2">
								<h4 class="text-base font-semibold">
									数据预览（{{ excelState.analysis.totalRows }} 行，展示前 {{ previewRows.length }} 行）
								</h4>
								<div class="overflow-x-auto rounded-md border border-(--ui-border)">
									<table class="w-full text-sm">
										<thead>
											<tr class="bg-(--ui-bg-muted)">
												<th
													v-for="(column, index) in excelColumns"
													:key="`header-${index}`"
													class="whitespace-nowrap px-3 py-2 text-left font-medium"
												>
													{{ column || `列 ${index + 1}` }}
												</th>
											</tr>
										</thead>
										<tbody>
											<tr
												v-for="(row, rowIndex) in previewRows"
												:key="`preview-${rowIndex}`"
												class="border-t border-(--ui-border)"
											>
												<td v-for="(cell, cellIndex) in row" :key="`cell-${rowIndex}-${cellIndex}`" class="px-3 py-2 align-top whitespace-pre-wrap">
													{{ cell }}
												</td>
											</tr>
										</tbody>
									</table>
								</div>
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
			</div>

			<div v-else class="space-y-6">
				<UCard class="bg-(--ui-bg)">
					<template #header>
						<div class="flex items-center gap-2">
							<Icon name="i-lucide-clipboard-list" class="size-5" />
							<h3 class="text-lg font-semibold">
								手动录入 DNAT 需求
							</h3>
						</div>
					</template>
					<div class="space-y-4">
						<div class="flex flex-wrap gap-3">
							<UButton icon="i-lucide-plus" @click="addManualRow">
								添加行
							</UButton>
							<UButton
								variant="outline"
								icon="i-lucide-trash"
								:disabled="manualRows.length <= 1"
								@click="clearManualRows"
							>
								清空
							</UButton>
						</div>
						<div class="overflow-x-auto rounded-md border border-(--ui-border)">
							<table class="w-full min-w-[720px] text-sm">
								<thead>
									<tr class="bg-(--ui-bg-muted)">
										<th class="px-3 py-2 text-left font-medium">
											协议
										</th>
										<th class="px-3 py-2 text-left font-medium">
											主机 IP
										</th>
										<th class="px-3 py-2 text-left font-medium">
											内网端口
										</th>
										<th class="px-3 py-2 text-left font-medium">
											外网 IP
										</th>
										<th class="px-3 py-2 text-left font-medium">
											外网端口
										</th>
										<th class="px-3 py-2 text-center font-medium">
											操作
										</th>
									</tr>
								</thead>
								<tbody>
									<tr v-for="(row, index) in manualRows" :key="`manual-${index}`" class="border-t border-(--ui-border)">
										<td class="px-3 py-2 align-top">
											<select
												v-model="row.protocol"
												class="select-compact"
											>
												<option value="TCP">
													TCP
												</option>
												<option value="UDP">
													UDP
												</option>
												<option value="ICMP">
													ICMP
												</option>
												<option value="ANY">
													ANY
												</option>
											</select>
										</td>
										<td class="px-3 py-2 align-top">
											<input
												v-model="row.internalIp"
												type="text"
												placeholder="192.168.1.100"
												class="w-full rounded-md border border-(--ui-border) bg-transparent px-2 py-1 text-sm focus:outline-none focus:ring-2 focus:ring-(--ui-primary)"
											>
										</td>
										<td class="px-3 py-2 align-top">
											<input
												v-model="row.internalPort"
												:type="row.protocol === 'ANY' ? 'text' : 'text'"
												:disabled="row.protocol === 'ANY' || row.protocol === 'ICMP'"
												placeholder="80 或 8000-8010"
												class="w-full rounded-md border border-(--ui-border) bg-transparent px-2 py-1 text-sm focus:outline-none focus:ring-2 focus:ring-(--ui-primary)"
											>
										</td>
										<td class="px-3 py-2 align-top">
											<textarea
												v-model="row.publicIp"
												:rows="2"
												placeholder="可填写多个 IP，支持换行"
												class="w-full resize-none rounded-md border border-(--ui-border) bg-transparent px-2 py-1 text-sm focus:outline-none focus:ring-2 focus:ring-(--ui-primary)"
											/>
										</td>
										<td class="px-3 py-2 align-top">
											<input
												v-model="row.publicPort"
												:type="row.protocol === 'ANY' ? 'text' : 'text'"
												:disabled="row.protocol === 'ANY' || row.protocol === 'ICMP'"
												placeholder="80 或 8000-8010"
												class="w-full rounded-md border border-(--ui-border) bg-transparent px-2 py-1 text-sm focus:outline-none focus:ring-2 focus:ring-(--ui-primary)"
											>
										</td>
										<td class="px-3 py-2 text-center align-top">
											<UButton
												variant="ghost"
												size="xs"
												icon="i-lucide-minus"
												:disabled="manualRows.length === 1"
												@click="removeManualRow(index)"
											/>
										</td>
									</tr>
								</tbody>
							</table>
						</div>
						<p class="text-sm text-(--ui-text-muted)">
							提示：协议为 ANY 或 ICMP 时无需填写端口；支持端口范围（例如 8000-8010），多个公网 IP 请使用换行。
						</p>
						<div class="flex flex-wrap gap-3">
							<UButton
								:loading="convertLoading"
								icon="i-lucide-check"
								@click="convertManualData"
							>
								校验并转换手动输入
							</UButton>
						</div>
					</div>
				</UCard>
			</div>

			<UCard v-if="convertErrors.length || natEntries.length" class="bg-(--ui-bg)">
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
							<li v-for="(error, index) in convertErrors" :key="`convert-error-${index}`">
								{{ error }}
							</li>
						</ul>
					</div>

					<div v-if="natEntries.length" class="space-y-3">
						<div class="flex flex-wrap items-center justify-between gap-3">
							<div>
								<p class="text-base font-semibold">
									已校验通过 {{ natEntries.length }} 条记录
								</p>
								<p class="text-sm text-(--ui-text-muted)">
									预览前 {{ Math.min(natEntries.length, 10) }} 条结果，生成命令时会全部使用。
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
						<div class="overflow-x-auto rounded-md border border-(--ui-border)">
							<table class="w-full text-sm">
								<thead>
									<tr class="bg-(--ui-bg-muted)">
										<th class="px-3 py-2 text-left font-medium">
											原始行
										</th>
										<th class="px-3 py-2 text-left font-medium">
											协议
										</th>
										<th class="px-3 py-2 text-left font-medium">
											内部地址
										</th>
										<th class="px-3 py-2 text-left font-medium">
											端口
										</th>
										<th class="px-3 py-2 text-left font-medium">
											公网地址
										</th>
										<th class="px-3 py-2 text-left font-medium">
											公网端口
										</th>
									</tr>
								</thead>
								<tbody>
									<tr v-for="entry in previewEntries" :key="`entry-${entry.rowIndex}`" class="border-t border-(--ui-border)">
										<td class="px-3 py-2 align-top">
											第 {{ entry.rowIndex }} 行
										</td>
										<td class="px-3 py-2 align-top">
											{{ entry.protocol }}
										</td>
										<td class="px-3 py-2 align-top">
											{{ entry.internalIp }}
										</td>
										<td class="px-3 py-2 align-top">
											<div v-if="entry.internalPortStart !== null">
												{{ formatPortRange(entry.internalPortStart, entry.internalPortEnd) }}
											</div>
										</td>
										<td class="px-3 py-2 align-top">
											<ul class="space-y-1">
												<li v-for="ip in entry.publicIps" :key="`pub-${entry.rowIndex}-${ip}`">
													{{ ip }}
												</li>
											</ul>
										</td>
										<td class="px-3 py-2 align-top">
											<div v-if="entry.publicPortStart !== null">
												{{ formatPortRange(entry.publicPortStart, entry.publicPortEnd) }}
											</div>
										</td>
									</tr>
								</tbody>
							</table>
						</div>
					</div>
				</div>
			</UCard>

			<UCard class="bg-(--ui-bg)">
				<template #header>
					<div class="flex items-center gap-2">
						<Icon name="i-lucide-sliders-horizontal" class="size-5" />
						<h3 class="text-lg font-semibold">
							设备与输出选项
						</h3>
					</div>
				</template>
				<div class="space-y-4">
					<div class="grid gap-4 md:grid-cols-2">
						<div class="space-y-2">
							<label class="text-sm font-semibold text-(--ui-text-muted)">
								设备类型
							</label>
							<URadioGroup v-model="deviceType" :items="deviceOptions" class="flex gap-4" />
						</div>
						<div class="space-y-2">
							<label class="text-sm font-semibold text-(--ui-text-muted)">
								弹性 IP 映射
							</label>
							<div class="flex items-center gap-3">
								<USwitch v-model="useElasticIp" />
								<span class="text-sm">
									{{ useElasticIp ? "启用" : "关闭" }}
								</span>
							</div>
						</div>
					</div>
					<div v-if="deviceType === 'h3c'" class="grid gap-3 md:grid-cols-[220px_1fr]">
						<label class="text-sm font-semibold text-(--ui-text-muted)">
							VRRP ID
						</label>
						<input
							v-model="vrrpId"
							type="number"
							min="1"
							max="65535"
							placeholder="请输入 H3C VRRP ID"
							class="w-full rounded-md border border-(--ui-border) bg-transparent px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-(--ui-primary)"
						>
					</div>
					<div>
						<NuxtLink to="/nat-batch/settings">
							<UButton
								variant="outline"
								size="sm"
								icon="i-lucide-external-link"
							>
								打开配置页面
							</UButton>
						</NuxtLink>
					</div>
				</div>
			</UCard>

			<UCard class="bg-(--ui-bg)">
				<template #header>
					<div class="flex items-center gap-2">
						<Icon name="i-lucide-terminal" class="size-5" />
						<h3 class="text-lg font-semibold">
							NAT 命令输出
						</h3>
					</div>
				</template>
				<div class="space-y-4">
					<div
						v-if="missingElasticIps.length"
						class="rounded-md border border-red-300 bg-red-50 px-4 py-3 text-sm text-red-700"
					>
						<p>
							以下内部 IP 未找到弹性 IP 映射：{{ missingElasticIps.join(", ") }}
						</p>
						<p class="mt-2">
							<NuxtLink class="underline" to="/nat-batch/settings">
								前往配置页面补全映射
							</NuxtLink>
						</p>
					</div>
					<UTextarea
						v-model="commandsPreview"
						:rows="12"
						class="font-mono text-sm"
						readonly
						placeholder="点击“生成 NAT 命令”后在此查看结果"
					/>
					<div class="flex flex-wrap gap-3">
						<UButton
							icon="i-lucide-cpu"
							:loading="generationLoading"
							:disabled="!natEntries.length"
							@click="generateCommands"
						>
							生成 NAT 命令
						</UButton>
						<UButton
							variant="outline"
							icon="i-lucide-copy"
							:disabled="!generatedCommands.length"
							@click="copyCommands"
						>
							复制到剪贴板
						</UButton>
						<UButton
							variant="outline"
							icon="i-lucide-save"
							:disabled="!generatedCommands.length"
							@click="saveCommandsToFile"
						>
							保存到文件
						</UButton>
					</div>
				</div>
			</UCard>

		</div>
	</LayoutTile>
</template>

<script lang="ts" setup>
	import { computed, reactive, ref, watch } from "vue";
	import type {
		ConvertResponse,
		DeviceType,
		ExcelAnalysis,
		GenerateNatCommandsResponse,
		ManualEntryRequest,
		NatEntry
	} from "~/types/nat-batch";
	import { extractErrorMessage } from "~/utils/error";

	definePageMeta({
		name: "NAT 批量生成",
		icon: "lucide:merge",
		description: "批量生成 NAT 配置命令",
		category: "firewall"
	});

	const requiredFields = ["协议", "主机IP", "内网端口", "外网IP", "外网端口"];

	type Mode = "excel" | "manual";

	interface ManualRow {
		protocol: ManualEntryRequest["protocol"]
		internalIp: string
		internalPort: string
		publicIp: string
		publicPort: string
	}

	const mode = ref<Mode>("excel");
	const modeOptions = [
		{ label: "Excel 导入", value: "excel" },
		{ label: "手动输入", value: "manual" }
	];

	const toast = useToast();

const excelState = reactive({
	filePath: "",
	analysis: null as ExcelAnalysis | null,
	selectedSheet: "",
	columnMapping: {} as Record<string, string>,
	previewRows: [] as string[][],
	isLoading: false
});

const convertLoading = ref(false);
const generationLoading = ref(false);
const convertErrors = ref<string[]>([]);
const natEntries = ref<NatEntry[]>([]);
const generatedCommands = ref<string[]>([]);
const missingElasticIps = ref<string[]>([]);

const manualRows = ref<ManualRow[]>([createManualRow()]);

const useElasticIp = ref(true);
const deviceType = ref<DeviceType>("huawei");
const vrrpId = ref("");

const excelColumns = computed(() => excelState.analysis?.columns ?? []);
const previewRows = computed(() => excelState.previewRows.slice(0, 10));
const previewEntries = computed(() => natEntries.value.slice(0, 10));
const isColumnMappingIncomplete = computed(() => {
	if (!excelState.analysis) return true;
	return requiredFields.some((field) => !excelState.columnMapping[field]);
});

const commandsPreview = computed(() => generatedCommands.value.join("\n"));

const deviceOptions = [
	{ label: "华为", value: "huawei" },
	{ label: "H3C", value: "h3c" }
];

const shouldIgnoreSheetChange = ref(false);

watch(
	() => excelState.selectedSheet,
	(sheet, oldSheet) => {
		if (shouldIgnoreSheetChange.value) {
			shouldIgnoreSheetChange.value = false;
			return;
		}
		if (!excelState.analysis || !excelState.filePath || sheet === oldSheet || !sheet) {
			return;
		}
		void analyzeExcel(sheet);
	}
);

watch(
	() => mode.value,
	() => {
		convertErrors.value = [];
		natEntries.value = [];
		generatedCommands.value = [];
		missingElasticIps.value = [];
	}
);

	function createManualRow(): ManualRow {
		return {
			protocol: "TCP",
			internalIp: "",
			internalPort: "",
			publicIp: "",
			publicPort: ""
		};
	}

	function resetColumnMapping() {
		if (!excelState.analysis) return;
		const mapping: Record<string, string> = {};
		requiredFields.forEach((field) => {
			mapping[field] = excelState.analysis?.suggestedMapping[field] ?? "";
		});
		excelState.columnMapping = mapping;
	}

	function clearExcelContext() {
		excelState.analysis = null;
		excelState.filePath = "";
		excelState.selectedSheet = "";
		excelState.columnMapping = {};
		excelState.previewRows = [];
		convertErrors.value = [];
		natEntries.value = [];
		generatedCommands.value = [];
		missingElasticIps.value = [];
	}

	function addManualRow() {
		manualRows.value.push(createManualRow());
	}

	function removeManualRow(index: number) {
		if (manualRows.value.length === 1) return;
		manualRows.value.splice(index, 1);
	}

	function clearManualRows() {
		manualRows.value = [createManualRow()];
	}

	async function handleSelectExcel() {
		try {
			const { open } = await import("@tauri-apps/plugin-dialog");
			const selected = await open({
				multiple: false,
				filters: [
					{ name: "Excel", extensions: ["xlsx", "xls"] }
				]
			});
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

		const analysis = await useTauriCoreInvoke<ExcelAnalysis>("process_excel_data", {
			request: {
				filePath: excelState.filePath,
				sheetName: normalizedSheet
			}
		});

		excelState.analysis = analysis;
		shouldIgnoreSheetChange.value = true;
		excelState.selectedSheet = analysis.selectedSheet;
		excelState.columnMapping = requiredFields.reduce<Record<string, string>>((acc, field) => {
			acc[field] = analysis.suggestedMapping[field] ?? "";
			return acc;
		}, {});
		excelState.previewRows = analysis.previewRows;
		convertErrors.value = [];
		natEntries.value = [];
		generatedCommands.value = [];
		missingElasticIps.value = [];
		toast.add({
			title: "Excel 加载成功",
			description: `检测到 ${analysis.totalRows} 行数据，正在自动解析`,
			color: "success"
		});
		await convertExcelData();
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
				description: "请为所有字段选择对应的 Excel 列。",
				color: "warning"
			});
			return;
	}
	convertLoading.value = true;
	try {
		const selectedSheet = excelState.selectedSheet && excelState.selectedSheet.trim().length > 0
			? excelState.selectedSheet
			: undefined;
		const response = await useTauriCoreInvoke<ConvertResponse>("convert_excel_to_entries", {
			request: {
				source: "excel",
				file_path: excelState.filePath,
				sheet_name: selectedSheet,
				header_row_index: excelState.analysis.headerRowIndex,
				column_mapping: excelState.columnMapping
			}
		});
		handleConvertResponse(response);
		toast.add({
			title: "Excel 数据解析成功",
			description: `有效记录 ${response.entries.length} 条`,
			color: "success"
			});
		} catch (error) {
			convertErrors.value = [extractErrorMessage(error)];
			natEntries.value = [];
			toast.add({
				title: "解析失败",
				description: extractErrorMessage(error),
				color: "error"
			});
		} finally {
			convertLoading.value = false;
		}
	}

	function sanitizeManualRows(): ManualEntryRequest[] {
		return manualRows.value
			.map((row) => ({
				protocol: row.protocol,
				internalIp: row.internalIp.trim(),
				internalPort: row.internalPort.trim() || undefined,
				publicIp: row.publicIp.trim(),
				publicPort: row.publicPort.trim() || undefined
			}))
			.filter((row) => row.protocol && row.internalIp && row.publicIp);
	}

	async function convertManualData() {
		const rows = sanitizeManualRows();
		if (!rows.length) {
			toast.add({
				title: "缺少有效数据",
				description: "请至少填写一条完整的手动输入记录。",
				color: "warning"
			});
			return;
		}
		convertLoading.value = true;
		try {
			const response = await useTauriCoreInvoke<ConvertResponse>("convert_excel_to_entries", {
				request: {
					source: "manual",
					rows
				}
			});
			handleConvertResponse(response);
			toast.add({
				title: "手动数据校验成功",
				description: `有效记录 ${response.entries.length} 条`,
				color: "success"
			});
		} catch (error) {
			convertErrors.value = [extractErrorMessage(error)];
			natEntries.value = [];
			toast.add({
				title: "校验失败",
				description: extractErrorMessage(error),
				color: "error"
			});
		} finally {
			convertLoading.value = false;
		}
	}

	function handleConvertResponse(response: ConvertResponse) {
		natEntries.value = response.entries;
		convertErrors.value = response.errors;
		generatedCommands.value = [];
		missingElasticIps.value = [];
	}

	function regeneratePreview() {
		// 通过克隆数组触发依赖更新
		natEntries.value = [...natEntries.value];
	}

	function formatPortRange(start: number | null, end: number | null) {
		if (start === null) return "";
		if (end === null || start === end) return String(start);
		return `${start}-${end}`;
	}

	async function generateCommands() {
		if (!natEntries.value.length) {
			toast.add({
				title: "无可生成的数据",
				description: "请先解析 Excel 或手动输入数据。",
				color: "warning"
			});
			return;
		}
		if (deviceType.value === "h3c") {
			const parsed = Number(vrrpId.value);
			if (!parsed || parsed <= 0 || parsed > 65535) {
				toast.add({
					title: "VRRP ID 无效",
					description: "H3C 模式下必须填写 1-65535 的 VRRP ID。",
					color: "warning"
				});
				return;
			}
		}

		generationLoading.value = true;
		try {
			const payload: Record<string, unknown> = {
				entries: natEntries.value,
				useElasticIp: useElasticIp.value,
				deviceType: deviceType.value
			};
			if (deviceType.value === "h3c") {
				payload.vrrpId = Number(vrrpId.value);
			}
			const result = await useTauriCoreInvoke<GenerateNatCommandsResponse>("generate_nat_commands", {
				request: payload
			});
			missingElasticIps.value = result.missingElasticIps;

			if (useElasticIp.value && result.missingElasticIps.length > 0) {
				generatedCommands.value = [];
				toast.add({
					title: "生成失败",
					description: `存在 ${result.missingElasticIps.length} 个内部 IP 缺少弹性 IP 映射，请前往配置页面补全。`,
					color: "warning"
				});
				return;
			}

			generatedCommands.value = result.commands;
			toast.add({
				title: "NAT 命令生成完成",
				description: `共生成 ${result.commands.length} 条命令`,
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

	async function copyCommands() {
		if (!generatedCommands.value.length) return;
		try {
			await navigator.clipboard.writeText(generatedCommands.value.join("\n"));
			toast.add({
				title: "复制成功",
				description: "命令已复制到剪贴板。",
				color: "success"
			});
		} catch (error) {
			toast.add({
				title: "复制失败",
				description: extractErrorMessage(error),
				color: "error"
			});
		}
	}

	async function saveCommandsToFile() {
		if (!generatedCommands.value.length) return;
		try {
			const { save } = await import("@tauri-apps/plugin-dialog");
			const path = await save({
				defaultPath: "nat_commands.txt",
				filters: [{ name: "Text", extensions: ["txt"] }]
			});
			if (!path) return;
			await useTauriCoreInvoke("export_nat_commands", {
				path,
				commands: generatedCommands.value
			});
			toast.add({
				title: "保存成功",
				description: `文件已保存到 ${path}`,
				color: "success"
			});
		} catch (error) {
			toast.add({
				title: "保存失败",
				description: extractErrorMessage(error),
				color: "error"
			});
		}
	}

	async function handleExportTemplate() {
		try {
			const { save } = await import("@tauri-apps/plugin-dialog");
			const path = await save({
				defaultPath: "NAT配置模板.xlsx",
				filters: [{ name: "Excel", extensions: ["xlsx"] }]
			});
			if (!path) return;
			await useTauriCoreInvoke("export_nat_template", { path });
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

</script>
