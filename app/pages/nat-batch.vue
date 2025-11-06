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
								<select
									v-model="excelState.selectedSheet"
									class="w-full rounded-md border border-(--ui-border) bg-transparent px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-(--ui-primary)"
								>
									<option v-for="sheet in excelState.analysis.sheetNames" :key="sheet" :value="sheet">
										{{ sheet }}
									</option>
								</select>
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
											class="w-full rounded-md border border-(--ui-border) bg-transparent px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-(--ui-primary)"
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
												class="w-full rounded-md border border-(--ui-border) bg-transparent px-2 py-1 text-sm"
											>
												<option value="TCP">
													TCP
												</option>
												<option value="UDP">
													UDP
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
												:disabled="row.protocol === 'ANY'"
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
												:disabled="row.protocol === 'ANY'"
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
						<UAlert
							variant="soft"
							color="primary"
							icon="i-lucide-info"
						>
							协议为 ANY 时无需填写端口；支持端口范围（例如 8000-8010），多个公网 IP 请使用换行。
						</UAlert>
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
					<UAlert
						v-if="convertErrors.length"
						variant="soft"
						color="warning"
						icon="i-lucide-alert-triangle"
					>
						<div class="space-y-1">
							<p class="font-medium">
								检测到 {{ convertErrors.length }} 条告警：
							</p>
							<ul class="list-disc pl-5 space-y-1">
								<li v-for="(error, index) in convertErrors" :key="`convert-error-${index}`">
									{{ error }}
								</li>
							</ul>
						</div>
					</UAlert>

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
											内部地址/端口
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
											<div class="space-y-1">
												<p>{{ entry.internalIp }}</p>
												<p v-if="entry.internalPortStart !== null">
													端口：{{ formatPortRange(entry.internalPortStart, entry.internalPortEnd) }}
												</p>
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
					<UAlert
						variant="soft"
						color="info"
						icon="i-lucide-lightbulb"
					>
						弹性 IP 映射启用后，内部地址会优先替换为映射的弹性 IP。请在下方“弹性 IP 管理”中维护映射关系。
					</UAlert>
				</div>
			</UCard>

			<UCard class="bg-(--ui-bg)">
				<template #header>
					<div class="flex items-center gap-2">
						<Icon name="i-lucide-cloud" class="size-5" />
						<h3 class="text-lg font-semibold">
							弹性 IP 管理
						</h3>
					</div>
				</template>
				<div class="space-y-4">
					<div class="grid gap-4 md:grid-cols-2">
						<div class="space-y-2">
							<label class="text-sm font-semibold text-(--ui-text-muted)">
								内部 IP
							</label>
							<input
								v-model="newMapping.internalIp"
								type="text"
								placeholder="192.168.1.100"
								class="w-full rounded-md border border-(--ui-border) bg-transparent px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-(--ui-primary)"
							>
						</div>
						<div class="space-y-2">
							<label class="text-sm font-semibold text-(--ui-text-muted)">
								弹性 IP
							</label>
							<input
								v-model="newMapping.elasticIp"
								type="text"
								placeholder="222.240.138.4"
								class="w-full rounded-md border border-(--ui-border) bg-transparent px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-(--ui-primary)"
							>
						</div>
					</div>
					<div class="flex flex-wrap gap-3">
						<UButton icon="i-lucide-plus" @click="addElasticMapping">
							添加/更新映射
						</UButton>
						<UButton
							variant="outline"
							size="sm"
							icon="i-lucide-rotate-ccw"
							@click="loadElasticMappings"
						>
							刷新
						</UButton>
					</div>
					<div class="space-y-3">
						<label class="text-sm font-semibold text-(--ui-text-muted)">
							批量导入
						</label>
						<UTextarea
							v-model="bulkMappingInput"
							:rows="4"
							placeholder="按行输入映射，例如：&#10;192.168.1.100 -> 222.240.138.4&#10;192.168.1.101 -> 222.240.138.5"
						/>
						<div class="flex flex-wrap items-center gap-3">
							<UCheckbox v-model="overwriteExisting" label="覆盖已存在的映射" />
							<UButton
								variant="soft"
								size="sm"
								icon="i-lucide-upload"
								:disabled="!bulkMappingInput.trim()"
								@click="bulkAddElasticMappings"
							>
								导入
							</UButton>
						</div>
					</div>
					<div class="overflow-x-auto rounded-md border border-(--ui-border)">
						<table class="w-full min-w-[480px] text-sm">
							<thead>
								<tr class="bg-(--ui-bg-muted)">
									<th class="px-3 py-2 text-left font-medium">
										内部 IP
									</th>
									<th class="px-3 py-2 text-left font-medium">
										弹性 IP
									</th>
									<th class="px-3 py-2 text-center font-medium">
										操作
									</th>
								</tr>
							</thead>
							<tbody>
								<tr
									v-for="mapping in elasticMappings"
									:key="`elastic-${mapping.internalIp}`"
									class="border-t border-(--ui-border)"
								>
									<td class="px-3 py-2">
										{{ mapping.internalIp }}
									</td>
									<td class="px-3 py-2">
										{{ mapping.elasticIp }}
									</td>
									<td class="px-3 py-2 text-center">
										<UButton
											variant="ghost"
											size="xs"
											icon="i-lucide-trash"
											@click="removeElasticMapping(mapping.internalIp)"
										/>
									</td>
								</tr>
								<tr v-if="!elasticMappings.length">
									<td colspan="3" class="px-3 py-4 text-center text-(--ui-text-muted)">
										暂无映射，请先添加。
									</td>
								</tr>
							</tbody>
						</table>
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
					<UAlert
						v-if="missingElasticIps.length"
						variant="soft"
						color="warning"
						icon="i-lucide-alert-circle"
					>
						以下内部 IP 未找到弹性 IP 映射：{{ missingElasticIps.join(", ") }}
					</UAlert>
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

			<UCard class="bg-(--ui-bg)">
				<template #header>
					<div class="flex items-center gap-2">
						<Icon name="i-lucide-globe-2" class="size-5" />
						<h3 class="text-lg font-semibold">
							运营商 IP 数据
						</h3>
					</div>
				</template>
				<div class="space-y-4">
					<div class="flex flex-wrap gap-3">
						<UButton
							:loading="ispUpdateLoading"
							icon="i-lucide-cloud-download"
							@click="updateIspDatabase"
						>
							从 GitHub 更新
						</UButton>
						<p v-if="ispSummary" class="text-sm text-(--ui-text-muted)">
							最近更新：电信 {{ ispSummary.dxCount }} 条，联通 {{ ispSummary.ltCount }} 条，移动 {{ ispSummary.ydCount }} 条，其他 {{ ispSummary.otherCount }} 条
						</p>
					</div>
					<div v-if="ispSummary" class="text-sm text-(--ui-text-muted)">
						数据保存位置：{{ ispSummary.savedPath }}
					</div>
				</div>
			</UCard>
		</div>
	</LayoutTile>
</template>

<script lang="ts" setup>
	import { computed, reactive, ref, watch } from "vue";
	import type {
		BulkElasticResult,
		ConvertResponse,
		DeviceType,
		ElasticMappingEntry,
		ExcelAnalysis,
		GenerateNatCommandsResponse,
		IspUpdateResult,
		ManualEntryRequest,
		NatEntry
	} from "~/types/nat-batch";
	import { extractErrorMessage } from "~/utils/error";

	definePageMeta({
		name: "NAT 批量生成",
		icon: "lucide:merge",
		description: "批量生成 NAT 配置命令",
		category: "tools"
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

	const elasticMappings = ref<ElasticMappingEntry[]>([]);
	const newMapping = reactive({ internalIp: "", elasticIp: "" });
	const bulkMappingInput = ref("");
	const overwriteExisting = ref(false);

	const ispUpdateLoading = ref(false);
	const ispSummary = ref<IspUpdateResult | null>(null);

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

	onMounted(() => {
		void loadElasticMappings();
	});

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
				description: `检测到 ${analysis.totalRows} 行数据`,
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
			generatedCommands.value = result.commands;
			missingElasticIps.value = result.missingElasticIps;
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

	async function loadElasticMappings() {
		try {
			const mappings = await useTauriCoreInvoke<ElasticMappingEntry[]>("get_all_elastic_mappings");
			elasticMappings.value = mappings;
		} catch (error) {
			toast.add({
				title: "加载映射失败",
				description: extractErrorMessage(error),
				color: "error"
			});
		}
	}

	async function addElasticMapping() {
		if (!newMapping.internalIp.trim() || !newMapping.elasticIp.trim()) {
			toast.add({
				title: "输入不完整",
				description: "请填写完整的内部 IP 和弹性 IP。",
				color: "warning"
			});
			return;
		}
		try {
			await useTauriCoreInvoke("add_elastic_ip_mapping", {
				request: {
					internalIp: newMapping.internalIp.trim(),
					elasticIp: newMapping.elasticIp.trim()
				}
			});
			await loadElasticMappings();
			newMapping.internalIp = "";
			newMapping.elasticIp = "";
			toast.add({
				title: "映射保存成功",
				color: "success"
			});
		} catch (error) {
			toast.add({
				title: "保存映射失败",
				description: extractErrorMessage(error),
				color: "error"
			});
		}
	}

	async function removeElasticMapping(internalIp: string) {
		try {
			await useTauriCoreInvoke("remove_elastic_ip_mapping", { internal_ip: internalIp });
			await loadElasticMappings();
			toast.add({
				title: "映射已删除",
				color: "success"
			});
		} catch (error) {
			toast.add({
				title: "删除失败",
				description: extractErrorMessage(error),
				color: "error"
			});
		}
	}

	async function bulkAddElasticMappings() {
		const entries = bulkMappingInput.value
			.split("\n")
			.map((line) => line.trim())
			.filter(Boolean)
			.map((line) => {
				const [left, right] = line.split("->").map((part) => part.trim());
				return left && right
					? { internalIp: left, elasticIp: right }
					: null;
			})
			.filter((entry): entry is { internalIp: string, elasticIp: string } => Boolean(entry));

		if (!entries.length) {
			toast.add({
				title: "无有效映射",
				description: "请按“内部IP -> 弹性IP”的格式填写映射。",
				color: "warning"
			});
			return;
		}

		try {
			const result = await useTauriCoreInvoke<BulkElasticResult>("bulk_add_elastic_ip_mappings", {
				request: {
					entries,
					overwriteExisting: overwriteExisting.value
				}
			});
			await loadElasticMappings();
			bulkMappingInput.value = "";
			toast.add({
				title: "批量导入完成",
				description: `新增 ${result.added} 条，更新 ${result.updated} 条，跳过 ${result.skipped} 条。`,
				color: "success"
			});
		} catch (error) {
			toast.add({
				title: "批量导入失败",
				description: extractErrorMessage(error),
				color: "error"
			});
		}
	}

	async function updateIspDatabase() {
		ispUpdateLoading.value = true;
		try {
			const summary = await useTauriCoreInvoke<IspUpdateResult>("update_isp_from_github");
			ispSummary.value = summary;
			toast.add({
				title: "运营商数据更新成功",
				description: `总计 ${summary.total} 条记录`,
				color: "success"
			});
		} catch (error) {
			toast.add({
				title: "更新失败",
				description: extractErrorMessage(error),
				color: "error"
			});
		} finally {
			ispUpdateLoading.value = false;
		}
	}

	function extractErrorMessage(error: unknown): string {
		if (typeof error === "string") return error;
		if (error instanceof Error) return error.message;
		if (error && typeof error === "object" && "message" in error && typeof (error as any).message === "string") {
			return (error as any).message;
		}
		return "发生未知错误";
	}
</script>
