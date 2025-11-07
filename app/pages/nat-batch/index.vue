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
					<UHeading :level="3" size="lg" class="font-semibold">
						数据来源
					</UHeading>
				</div>
				</template>
				<div class="space-y-3">
					<URadioGroup
						v-model="mode"
						:items="modeOptions"
						orientation="horizontal"
						class="flex flex-wrap gap-6"
					/>
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
						<UHeading :level="3" size="lg" class="font-semibold">
							Excel 数据
						</UHeading>
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
							<UHeading :level="4" size="md" class="font-semibold">
								列映射
							</UHeading>
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
									v-for="field in requiredFields"
									:key="field"
									class="flex flex-col gap-2 min-w-[160px] flex-1"
								>
									<div class="flex items-center justify-between gap-2">
										<span class="text-xs font-semibold text-(--ui-text-muted)">
											{{ field }}
										</span>
										<div v-if="isFieldMapped(field)" class="flex items-center gap-1 text-xs text-green-600">
											<Icon name="i-lucide-check" class="size-3" />
											<span>已映射</span>
										</div>
										<div v-else class="flex items-center gap-1 text-xs text-gray-400">
											<Icon name="i-lucide-circle" class="size-3" />
											<span>未映射</span>
										</div>
									</div>
									<USelect
										v-model="excelState.columnMapping[field]"
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
								<UHeading :level="4" size="md" class="font-semibold">
									数据预览（{{ excelState.analysis.totalRows }} 行，展示前 {{ previewRows.length }} 行）
								</UHeading>
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
			</div>

			<div v-else class="space-y-6">
			<UCard class="bg-(--ui-bg)">
				<template #header>
					<div class="flex items-center gap-2">
						<Icon name="i-lucide-clipboard-list" class="size-5" />
						<UHeading :level="3" size="lg" class="font-semibold">
							手动录入 DNAT 需求
						</UHeading>
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
						<div class="overflow-x-auto rounded-lg border border-(--ui-border) shadow-sm">
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
											<USelect
												v-model="row.protocol"
												:items="['TCP', 'UDP', 'ICMP', 'ANY']"
												placeholder="选择协议"
												size="sm"
												class="w-full"
											/>
										</td>
										<td class="px-3 py-2 align-top">
											<UInput
												v-model="row.internalIp"
												placeholder="192.168.1.100"
												size="sm"
											/>
										</td>
										<td class="px-3 py-2 align-top">
											<UInput
												v-model="row.internalPort"
												placeholder="80 或 8000-8010"
												:disabled="row.protocol === 'ANY' || row.protocol === 'ICMP'"
												size="sm"
											/>
										</td>
										<td class="px-3 py-2 align-top">
											<UInput
												v-model="row.publicIp"
												placeholder="222.240.138.4"
												size="sm"
											/>
										</td>
										<td class="px-3 py-2 align-top">
											<UInput
												v-model="row.publicPort"
												placeholder="80 或 8000-8010"
												:disabled="row.protocol === 'ANY' || row.protocol === 'ICMP'"
												size="sm"
											/>
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
							提示：协议为 ANY 或 ICMP 时无需填写端口；支持端口范围（例如 8000-8010）。
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
					<UHeading :level="3" size="lg" class="font-semibold">
						数据校验结果
					</UHeading>
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
						<li v-for="(error, index) in convertErrors" :key="`convert-error-${index}`">
							{{ error }}
						</li>
					</ul>
				</UAlert>

					<div v-if="natEntries.length" class="space-y-4">
						<div class="flex flex-wrap items-center justify-between gap-4 p-4 bg-green-50 border border-green-200 rounded-lg">
							<div class="flex items-center gap-2">
								<Icon name="i-lucide-check-circle" class="size-5 text-green-600" />
								<div>
									<p class="text-base font-semibold text-green-800">
										已校验通过 {{ natEntries.length }} 条记录
									</p>
									<p class="text-sm text-green-600">
										预览前 {{ Math.min(natEntries.length, 10) }} 条结果，生成命令时会全部使用。
									</p>
								</div>
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
							:columns="validationColumns"
							:data="previewEntries"
							:row-key="row => row.rowIndex"
							class="w-full"
						>
							<template #rowIndex-cell="{ row }">
								第 {{ row.original.rowIndex }} 行
							</template>
							<template #publicIps-cell="{ row }">
								<ul class="space-y-1">
									<li v-for="ip in row.original.publicIps" :key="`pub-${row.original.rowIndex}-${ip}`">
										{{ ip }}
									</li>
								</ul>
							</template>
						</UTable>
					</div>
				</div>
			</UCard>

		<UCard class="bg-(--ui-bg)">
			<template #header>
				<div class="flex items-center gap-2">
					<Icon name="i-lucide-sliders-horizontal" class="size-5" />
					<UHeading :level="3" size="lg" class="font-semibold">
						设备与输出选项
					</UHeading>
				</div>
			</template>

			<div class="space-y-6">
				<!-- 主要配置区域 -->
				<div class="grid gap-6 md:grid-cols-2">
					<!-- 设备类型 -->
					<div class="space-y-3">
						<label class="text-sm font-semibold text-(--ui-text-muted)">
							设备类型
						</label>
						<URadioGroup
							v-model="deviceType"
							:items="deviceOptions"
							orientation="horizontal"
							class="flex flex-wrap gap-4"
						/>
					</div>

					<!-- 运营商数据来源 -->
					<div class="space-y-3">
						<label class="text-sm font-semibold text-(--ui-text-muted)">
							运营商数据来源
						</label>
						<URadioGroup
							v-model="ispSource"
							:items="ispSourceOptions"
							orientation="horizontal"
							class="flex flex-wrap gap-4"
						/>
						<p class="text-xs text-(--ui-text-muted) mt-2">
							GitHub 无法访问时，可切换为在线 API 查询（依赖公网访问）。
						</p>
					</div>
				</div>

				<!-- 弹性 IP 映射配置 -->
				<div class="p-4 bg-(--ui-bg-muted) rounded-lg border border-(--ui-border)">
					<div class="flex items-start justify-between gap-4">
						<div class="space-y-3 flex-1">
							<div class="flex items-center gap-3">
								<Icon name="i-lucide-globe" class="size-4 text-(--ui-text-muted)" />
								<label class="text-sm font-semibold text-(--ui-text)">
									弹性 IP 映射
								</label>
							</div>
							<div class="flex items-center gap-3">
								<USwitch v-model="useElasticIp" size="sm" />
								<span class="text-sm text-(--ui-text)">
									{{ useElasticIp ? "已启用" : "已关闭" }}
								</span>
							</div>
						</div>
						<NuxtLink to="/nat-batch/settings">
							<UButton
								variant="outline"
								size="sm"
								icon="i-lucide-external-link"
							>
								配置映射
							</UButton>
						</NuxtLink>
					</div>
				</div>

				<!-- H3C 专用配置 -->
				<div v-if="deviceType === 'h3c'" class="p-4 bg-amber-50 border border-amber-200 rounded-lg">
					<div class="flex items-center gap-2 mb-3">
						<Icon name="i-lucide-settings" class="size-4 text-amber-600" />
						<label class="text-sm font-semibold text-amber-800">
							H3C 设备专用配置
						</label>
					</div>
					<div class="grid gap-3 md:grid-cols-[200px_1fr]">
						<label class="text-sm font-medium text-amber-700">
							VRRP ID
						</label>
						<div class="flex items-center gap-3">
							<UInput
								v-model="vrrpId"
								type="number"
								:min="1"
								:max="65535"
								placeholder="请输入 VRRP ID（1-65535）"
								class="flex-1"
								size="sm"
							/>
							<span class="text-xs text-amber-600">
								留空表示不配置 VRRP
							</span>
						</div>
					</div>
				</div>
			</div>
		</UCard>

		<UCard class="bg-(--ui-bg)">
			<template #header>
				<div class="flex items-center gap-2">
					<Icon name="i-lucide-terminal" class="size-5" />
					<UHeading :level="3" size="lg" class="font-semibold">
						NAT 命令输出
					</UHeading>
				</div>
			</template>
			<div class="space-y-4">
				<UAlert
					v-if="missingElasticIps.length"
					variant="danger"
					icon="i-lucide-alert-triangle"
					title="缺少弹性 IP 映射"
				>
					<p>
						以下内部 IP 未找到弹性 IP 映射：{{ missingElasticIps.join(", ") }}
					</p>
					<template #actions>
						<UButton to="/nat-batch/settings" variant="outline" size="sm">
							前往配置页面补全映射
						</UButton>
					</template>
				</UAlert>
					<UTextarea
						v-model="commandsPreview"
						:rows="12"
						class="font-mono text-sm"
						readonly
						placeholder="点击“生成 NAT 命令”后在此查看结果"
					/>
					<div class="flex flex-wrap items-center gap-3">
						<div class="flex items-center gap-2">
							<UButton
								icon="i-lucide-cpu"
								:loading="generationLoading"
								:disabled="!natEntries.length"
								@click="generateCommands"
								size="md"
							>
								生成 NAT 命令
							</UButton>
						</div>
						<div class="flex items-center gap-2">
							<UButton
								variant="outline"
								icon="i-lucide-copy"
								:disabled="!generatedCommands.length"
								@click="copyCommands"
								size="sm"
							>
								复制
							</UButton>
							<UButton
								variant="outline"
								icon="i-lucide-save"
								:disabled="!generatedCommands.length"
								@click="saveCommandsToFile"
								size="sm"
							>
								保存
							</UButton>
						</div>
					</div>
				</div>
			</UCard>
		</div>
	</LayoutTile>
</template>

<script lang="ts" setup>
	import type {
		ConvertResponse,
		DeviceType,
		ExcelAnalysis,
		GenerateNatCommandsResponse,
		ManualEntryRequest,
		NatEntry,
		IspSource
	} from "~/types/nat-batch";
	import { computed, reactive, ref, watch } from "vue";
	import { extractErrorMessage } from "~/utils/error";

	definePageMeta({
		name: "NAT 批量生成",
		icon: "lucide:merge",
		description: "批量生成 NAT 配置命令",
		category: "firewall"
	});

	const requiredFields = ["协议", "主机IP", "内网端口", "外网IP", "外网端口"];
	const UNMAPPED_VALUE = "__unmapped__";

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
		columnMapping: requiredFields.reduce<Record<string, string>>((acc, field) => {
			acc[field] = UNMAPPED_VALUE;
			return acc;
		}, {}),
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
    const ispSource = ref<IspSource>("local");

	const excelColumns = computed(() => excelState.analysis?.columns ?? []);
	const previewRows = computed(() => excelState.previewRows.slice(0, 10));

	// 列映射选项
	const columnOptions = computed(() => [
		{ label: '未选择', value: UNMAPPED_VALUE },
		...excelColumns.value.map((col) => ({ label: col || '(空列)', value: col || UNMAPPED_VALUE }))
	]);

	// 检查字段是否已映射
	function isFieldMapped(field: string) {
		const value = excelState.columnMapping[field];
		return value && value !== UNMAPPED_VALUE;
	}

	const excelPreviewColumns = computed(() =>
		excelColumns.value.map((column, index) => ({
			id: `col_${index}`,
			accessorKey: `col_${index}`,
			header: column || `列 ${index + 1}`
		}))
	);

	const excelPreviewRows = computed(() => {
		const _columns = excelColumns.value;
		return previewRows.value.map((row) => {
			const rowObj: Record<string, string> = {};
			row.forEach((cell, index) => {
				rowObj[`col_${index}`] = cell;
			});
			return rowObj;
		});
	});

const validationColumns = computed(() => [
	{ id: "rowIndex", accessorKey: "rowIndex", header: "原始行" },
	{ id: "protocol", accessorKey: "protocol", header: "协议" },
	{ id: "internalIp", accessorKey: "internalIp", header: "内部地址" },
	{ id: "internalPort", accessorKey: "internalPortDisplay", header: "端口" },
	{ id: "publicIps", accessorKey: "publicIps", header: "公网地址" },
	{ id: "publicPort", accessorKey: "publicPortDisplay", header: "公网端口" }
]);
const previewEntries = computed(() =>
	natEntries.value.slice(0, 10).map((entry) => ({
		...entry,
		internalPortDisplay: formatPortRange(entry.internalPortStart, entry.internalPortEnd),
		publicPortDisplay: formatPortRange(entry.publicPortStart, entry.publicPortEnd)
	}))
);
const isColumnMappingIncomplete = computed(() => {
	if (!excelState.analysis) return true;
	return requiredFields.some((field) => {
		const value = excelState.columnMapping[field];
		return !value || value === UNMAPPED_VALUE;
	});
});

	const commandsPreview = computed(() => generatedCommands.value.join("\n"));

	const deviceOptions = [
		{ label: "华为", value: "huawei" },
		{ label: "H3C", value: "h3c" }
	];

	const ispSourceOptions = [
		{ label: "离线（本地 YAML 数据）", value: "local" },
		{ label: "在线 API 查询", value: "online" }
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

	watch(
		() => ispSource.value,
		() => {
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
			mapping[field] = excelState.analysis?.suggestedMapping[field] ?? UNMAPPED_VALUE;
		});
		excelState.columnMapping = mapping;
	}

	function clearExcelContext() {
		excelState.analysis = null;
		excelState.filePath = "";
		excelState.selectedSheet = "";
		excelState.columnMapping = requiredFields.reduce<Record<string, string>>((acc, field) => {
			acc[field] = UNMAPPED_VALUE;
			return acc;
		}, {});
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
			acc[field] = analysis.suggestedMapping[field] ?? UNMAPPED_VALUE;
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
		const normalizedMapping = Object.fromEntries(
			requiredFields.map((field) => [
				field,
				excelState.columnMapping[field] === UNMAPPED_VALUE ? "" : excelState.columnMapping[field]
			])
		);
		const response = await useTauriCoreInvoke<ConvertResponse>("convert_excel_to_entries", {
			request: {
				source: "excel",
				file_path: excelState.filePath,
				sheet_name: selectedSheet,
				header_row_index: excelState.analysis.headerRowIndex,
				column_mapping: normalizedMapping
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
				protocol: (() => {
					const candidate = row.protocol.trim().toUpperCase();
					return ["TCP", "UDP", "ICMP", "ANY"].includes(candidate) ? candidate : "TCP";
				})(),
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
		let parsedVrrp: number | null = null;
		if (deviceType.value === "h3c" && vrrpId.value.trim()) {
			const numeric = Number(vrrpId.value);
			if (!Number.isInteger(numeric) || numeric <= 0 || numeric > 65535) {
				toast.add({
					title: "VRRP ID 无效",
					description: "请输入 1-65535 范围内的 VRRP ID，或留空表示不配置。",
					color: "warning"
				});
				return;
			}
			parsedVrrp = numeric;
		}

		generationLoading.value = true;
		try {
			const payload: Record<string, unknown> = {
				entries: natEntries.value,
				useElasticIp: useElasticIp.value,
				deviceType: deviceType.value,
				ispSource: ispSource.value
			};
			if (deviceType.value === "h3c" && parsedVrrp !== null) {
				payload.vrrpId = parsedVrrp;
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
